/// Voice pipeline: HTTP upload of WAV audio, server response parsing
///
/// Sends recorded WAV audio to a voice processing server that performs:
/// 1. Speech-to-text (Whisper)
/// 2. Intent parsing (LLM) — extracts task name and recurrence
/// 3. Returns a structured JSON for task creation
extern crate alloc;

use alloc::string::String;

use esp_idf_svc::http::client::{Configuration as HttpClientConfig, EspHttpConnection};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::models::RecurrenceType;

/// Voice command action returned by the server (task creation only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAction {
    /// Action type: "create" or "none"
    #[serde(default = "default_action")]
    pub action: String,
    /// Task name extracted from speech
    #[serde(default)]
    pub task_name: String,
    /// Recurrence in days (e.g. 3 = every 3 days, 7 = weekly, 30 = monthly)
    #[serde(default)]
    pub recurrence_days: Option<u32>,
    /// Human-readable message to show on screen
    #[serde(default)]
    pub message: String,
}

fn default_action() -> String {
    String::from("none")
}

impl VoiceAction {
    /// Parse recurrence_days into a RecurrenceType and value
    pub fn recurrence(&self) -> (RecurrenceType, u32) {
        match self.recurrence_days {
            Some(days) if days >= 365 && days % 365 == 0 => {
                (RecurrenceType::Yearly, days / 365)
            }
            Some(days) if days >= 30 && days % 30 == 0 => {
                (RecurrenceType::Monthly, days / 30)
            }
            Some(days) if days >= 7 && days % 7 == 0 => {
                (RecurrenceType::Weekly, days / 7)
            }
            Some(days) => (RecurrenceType::Daily, days.max(1)),
            None => (RecurrenceType::Daily, 1),
        }
    }
}

/// Send WAV audio to the voice processing server and receive a parsed action.
///
/// The server URL is configured in config::VOICE_SERVER_URL.
/// Audio is sent as raw body with Content-Type: audio/wav.
/// Response is JSON matching the VoiceAction struct.
pub fn send_audio_to_server(
    wav_data: &[u8],
    _task_context: &str,
) -> Result<VoiceAction, VoiceError> {
    log::info!(
        "Sending {}KB audio to voice server: {}",
        wav_data.len() / 1024,
        config::VOICE_SERVER_URL
    );

    let url = String::from(config::VOICE_SERVER_URL);

    let http_config = HttpClientConfig {
        buffer_size: Some(2048),
        buffer_size_tx: Some(1024),
        timeout: Some(core::time::Duration::from_secs(30)),
        ..Default::default()
    };

    let mut connection = EspHttpConnection::new(&http_config)
        .map_err(|e| VoiceError::Connection(alloc::format!("{}", e)))?;

    // Prepare headers
    let content_length = alloc::format!("{}", wav_data.len());
    let headers = [
        ("Content-Type", "audio/wav"),
        ("Content-Length", content_length.as_str()),
    ];

    // Initiate POST request
    connection
        .initiate_request(esp_idf_svc::http::Method::Post, &url, &headers)
        .map_err(|e| VoiceError::Connection(alloc::format!("initiate: {}", e)))?;

    // Write WAV body in chunks to avoid memory issues
    let chunk_size = 1024;
    let mut offset = 0;
    while offset < wav_data.len() {
        let end = (offset + chunk_size).min(wav_data.len());
        connection
            .write(&wav_data[offset..end])
            .map_err(|e| VoiceError::Connection(alloc::format!("write: {}", e)))?;
        offset = end;
    }

    // Submit and get response
    connection
        .initiate_response()
        .map_err(|e| VoiceError::Connection(alloc::format!("response: {}", e)))?;

    let status = connection.status();
    log::info!("Voice server responded with status: {}", status);

    if status != 200 {
        return Err(VoiceError::ServerError(status));
    }

    // Read response body
    let mut response_buf = [0u8; 2048];
    let mut total_read = 0;

    loop {
        match connection.read(&mut response_buf[total_read..]) {
            Ok(0) => break,
            Ok(n) => {
                total_read += n;
                if total_read >= response_buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let response_str = core::str::from_utf8(&response_buf[..total_read])
        .map_err(|_| VoiceError::ParseError(String::from("Invalid UTF-8 in response")))?;

    log::info!("Voice server response: {}", response_str);

    // Parse JSON response
    let action: VoiceAction = serde_json::from_str(response_str)
        .map_err(|e| VoiceError::ParseError(alloc::format!("JSON parse: {}", e)))?;

    Ok(action)
}


/// Send a 5-second audio chunk to the server for transcription only.
///
/// Streams the WAV header + PCM data directly from the AudioBuffer
/// without allocating a full WAV copy (saves ~160KB of heap).
/// Returns the transcript text for this chunk.
pub fn send_chunk_to_server(
    audio_buf: &crate::microphone::AudioBuffer,
) -> Result<String, VoiceError> {
    log::info!(
        "Sending {}KB audio chunk to: {}",
        audio_buf.pcm_data.len() / 1024,
        config::VOICE_CHUNK_URL
    );

    let url = String::from(config::VOICE_CHUNK_URL);

    let http_config = HttpClientConfig {
        buffer_size: Some(2048),
        buffer_size_tx: Some(1024),
        timeout: Some(core::time::Duration::from_secs(30)),
        ..Default::default()
    };

    let mut connection = EspHttpConnection::new(&http_config)
        .map_err(|e| VoiceError::Connection(alloc::format!("{}", e)))?;

    // Build WAV header (44 bytes) — no full copy needed
    let wav_header = audio_buf.wav_header();
    let content_length = wav_header.len() + audio_buf.pcm_data.len();
    let content_length_str = alloc::format!("{}", content_length);

    let headers = [
        ("Content-Type", "audio/wav"),
        ("Content-Length", content_length_str.as_str()),
    ];

    connection
        .initiate_request(esp_idf_svc::http::Method::Post, &url, &headers)
        .map_err(|e| VoiceError::Connection(alloc::format!("initiate: {}", e)))?;

    // Write WAV header first
    connection
        .write(&wav_header)
        .map_err(|e| VoiceError::Connection(alloc::format!("write hdr: {}", e)))?;

    // Stream PCM data directly from audio_buf (no extra allocation)
    let chunk_size = 1024;
    let mut offset = 0;
    while offset < audio_buf.pcm_data.len() {
        let end = (offset + chunk_size).min(audio_buf.pcm_data.len());
        connection
            .write(&audio_buf.pcm_data[offset..end])
            .map_err(|e| VoiceError::Connection(alloc::format!("write: {}", e)))?;
        offset = end;
    }

    connection
        .initiate_response()
        .map_err(|e| VoiceError::Connection(alloc::format!("response: {}", e)))?;

    let status = connection.status();
    log::info!("Chunk server responded with status: {}", status);

    if status != 200 {
        return Err(VoiceError::ServerError(status));
    }

    // Read response — expecting {"transcript": "..."}
    let mut response_buf = [0u8; 2048];
    let mut total_read = 0;

    loop {
        match connection.read(&mut response_buf[total_read..]) {
            Ok(0) => break,
            Ok(n) => {
                total_read += n;
                if total_read >= response_buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let response_str = core::str::from_utf8(&response_buf[..total_read])
        .map_err(|_| VoiceError::ParseError(String::from("Invalid UTF-8 in chunk response")))?;

    log::info!("Chunk response: {}", response_str);

    // Parse transcript from JSON
    #[derive(Deserialize)]
    struct ChunkResponse {
        #[serde(default)]
        transcript: String,
    }

    let resp: ChunkResponse = serde_json::from_str(response_str)
        .map_err(|e| VoiceError::ParseError(alloc::format!("chunk JSON: {}", e)))?;

    Ok(resp.transcript)
}

/// Send the joined transcript to the server for LLM task extraction.
///
/// This is the second step of chunked recording: all chunk transcripts
/// have been joined into a single string, and we ask the LLM to extract
/// the task name and recurrence.
pub fn finalize_voice(transcript: &str) -> Result<VoiceAction, VoiceError> {
    log::info!(
        "Finalizing voice with {} chars of transcript: {}",
        transcript.len(),
        config::VOICE_COMPLETE_URL
    );

    let url = String::from(config::VOICE_COMPLETE_URL);

    let http_config = HttpClientConfig {
        buffer_size: Some(2048),
        buffer_size_tx: Some(1024),
        timeout: Some(core::time::Duration::from_secs(30)),
        ..Default::default()
    };

    let mut connection = EspHttpConnection::new(&http_config)
        .map_err(|e| VoiceError::Connection(alloc::format!("{}", e)))?;

    // Build JSON body — escape quotes in transcript
    let escaped = transcript.replace('\\', "\\\\").replace('"', "\\\"");
    let body = alloc::format!(r#"{{"transcript":"{}"}}"#, escaped);
    let content_length_str = alloc::format!("{}", body.len());

    let headers = [
        ("Content-Type", "application/json"),
        ("Content-Length", content_length_str.as_str()),
    ];

    connection
        .initiate_request(esp_idf_svc::http::Method::Post, &url, &headers)
        .map_err(|e| VoiceError::Connection(alloc::format!("initiate: {}", e)))?;

    connection
        .write(body.as_bytes())
        .map_err(|e| VoiceError::Connection(alloc::format!("write: {}", e)))?;

    connection
        .initiate_response()
        .map_err(|e| VoiceError::Connection(alloc::format!("response: {}", e)))?;

    let status = connection.status();
    log::info!("Finalize server responded with status: {}", status);

    if status != 200 {
        return Err(VoiceError::ServerError(status));
    }

    // Read response body
    let mut response_buf = [0u8; 2048];
    let mut total_read = 0;

    loop {
        match connection.read(&mut response_buf[total_read..]) {
            Ok(0) => break,
            Ok(n) => {
                total_read += n;
                if total_read >= response_buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    let response_str = core::str::from_utf8(&response_buf[..total_read])
        .map_err(|_| VoiceError::ParseError(String::from("Invalid UTF-8 in finalize response")))?;

    log::info!("Finalize response: {}", response_str);

    let action: VoiceAction = serde_json::from_str(response_str)
        .map_err(|e| VoiceError::ParseError(alloc::format!("finalize JSON: {}", e)))?;

    Ok(action)
}

/// Errors that can occur during voice processing
#[derive(Debug)]
pub enum VoiceError {
    Connection(String),
    ServerError(u16),
    ParseError(String),
}

impl core::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VoiceError::Connection(msg) => write!(f, "Connection error: {}", msg),
            VoiceError::ServerError(code) => write!(f, "Server error: HTTP {}", code),
            VoiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}
