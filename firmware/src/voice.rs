/// Voice pipeline: HTTP upload of WAV audio, server response parsing
///
/// Sends recorded WAV audio to a voice processing server that performs:
/// 1. Speech-to-text (Whisper)
/// 2. Intent parsing (LLM)
/// 3. Returns a structured JSON action
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use esp_idf_svc::http::client::{Configuration as HttpClientConfig, EspHttpConnection};
use serde::{Deserialize, Serialize};

use crate::config;
use crate::models::RecurrenceType;

/// Voice command action returned by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAction {
    /// Action type: "create", "update", "complete", "delete", "none"
    pub action: String,
    /// Task name (for create/update)
    #[serde(default)]
    pub task_name: String,
    /// Recurrence in days (for create/update) - server converts to appropriate type
    #[serde(default)]
    pub recurrence_days: Option<u32>,
    /// Human-readable message to show on screen
    #[serde(default)]
    pub message: String,
    /// Task ID to act on (for update/complete/delete, matched by server or name)
    #[serde(default)]
    pub task_id: Option<u32>,
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
    task_context: &str,
) -> Result<VoiceAction, VoiceError> {
    log::info!(
        "Sending {}KB audio to voice server: {}",
        wav_data.len() / 1024,
        config::VOICE_SERVER_URL
    );

    // Build the URL with task context as query param
    let url = if task_context.is_empty() {
        String::from(config::VOICE_SERVER_URL)
    } else {
        // URL-encode is overkill here; the server handles raw query strings
        alloc::format!("{}?context={}", config::VOICE_SERVER_URL, urlencode(task_context))
    };

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

/// Build a task context string from the current task list for the server.
/// This helps the LLM understand what tasks exist for update/complete commands.
pub fn build_task_context(tasks: &[crate::models::Task]) -> String {
    if tasks.is_empty() {
        return String::from("No tasks exist yet.");
    }

    let mut ctx = String::from("Current tasks: ");
    for (i, task) in tasks.iter().take(20).enumerate() {
        if i > 0 {
            ctx.push_str(", ");
        }
        ctx.push_str(&alloc::format!(
            "{}(id={},every {} {})",
            task.name,
            task.id,
            task.recurrence_value,
            task.recurrence_type.as_str()
        ));
    }
    ctx
}

/// Minimal URL encoding for the context query parameter
fn urlencode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.bytes() {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(c as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push('%');
                result.push(to_hex_digit(c >> 4));
                result.push(to_hex_digit(c & 0x0F));
            }
        }
    }
    result
}

fn to_hex_digit(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'A' + n - 10) as char,
        _ => '0',
    }
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
