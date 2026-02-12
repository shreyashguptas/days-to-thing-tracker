/// INMP441 I2S microphone driver
///
/// Records audio via I2S RX at 16 kHz, 16-bit mono.
/// The INMP441 outputs 32-bit frames; we right-shift to get 16-bit PCM.
/// Includes WAV header generation for the recorded buffer.
extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use esp_idf_hal::i2s::config::{
    Config, DataBitWidth, SlotMode, StdClkConfig, StdConfig, StdGpioConfig, StdSlotConfig,
};
use esp_idf_hal::i2s::{I2sDriver, I2sRx};
use esp_idf_hal::peripheral::Peripheral;

use crate::config;

/// Audio buffer holding recorded PCM data
pub struct AudioBuffer {
    /// Raw 16-bit PCM samples (little-endian)
    pub pcm_data: Vec<u8>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Bits per sample (16)
    pub bits_per_sample: u16,
    /// Number of channels (1 = mono)
    pub channels: u16,
}

impl AudioBuffer {
    /// Create a new empty audio buffer pre-allocated for initial recording capacity.
    /// The Vec will grow beyond this if the user records longer.
    pub fn new() -> Self {
        // 16 kHz * 2 bytes * initial_seconds = initial capacity
        let max_bytes =
            config::VOICE_SAMPLE_RATE * 2 * config::VOICE_INITIAL_BUF_SECS;
        Self {
            pcm_data: Vec::with_capacity(max_bytes as usize),
            sample_rate: config::VOICE_SAMPLE_RATE,
            bits_per_sample: 16,
            channels: 1,
        }
    }

    /// Clear the buffer for a new recording
    pub fn clear(&mut self) {
        self.pcm_data.clear();
    }

    /// Get the duration of recorded audio in seconds
    pub fn duration_secs(&self) -> f32 {
        let bytes_per_sample = (self.bits_per_sample / 8) as usize * self.channels as usize;
        if bytes_per_sample == 0 || self.sample_rate == 0 {
            return 0.0;
        }
        self.pcm_data.len() as f32 / (self.sample_rate as f32 * bytes_per_sample as f32)
    }

    /// Generate a 44-byte WAV header for the current PCM data.
    /// Used by chunked streaming to avoid allocating a full WAV copy —
    /// write this header first, then stream pcm_data directly.
    pub fn wav_header(&self) -> Vec<u8> {
        let data_size = self.pcm_data.len() as u32;
        let byte_rate =
            self.sample_rate * self.channels as u32 * (self.bits_per_sample / 8) as u32;
        let block_align = self.channels * (self.bits_per_sample / 8);
        let file_size = 36 + data_size;

        let mut hdr = Vec::with_capacity(44);
        hdr.extend_from_slice(b"RIFF");
        hdr.extend_from_slice(&file_size.to_le_bytes());
        hdr.extend_from_slice(b"WAVE");
        hdr.extend_from_slice(b"fmt ");
        hdr.extend_from_slice(&16u32.to_le_bytes());
        hdr.extend_from_slice(&1u16.to_le_bytes());
        hdr.extend_from_slice(&self.channels.to_le_bytes());
        hdr.extend_from_slice(&self.sample_rate.to_le_bytes());
        hdr.extend_from_slice(&byte_rate.to_le_bytes());
        hdr.extend_from_slice(&block_align.to_le_bytes());
        hdr.extend_from_slice(&self.bits_per_sample.to_le_bytes());
        hdr.extend_from_slice(b"data");
        hdr.extend_from_slice(&data_size.to_le_bytes());
        hdr
    }

    /// Encode the PCM buffer as a complete WAV file (44-byte header + PCM data)
    pub fn to_wav(&self) -> Vec<u8> {
        let data_size = self.pcm_data.len() as u32;
        let byte_rate = self.sample_rate * self.channels as u32 * (self.bits_per_sample / 8) as u32;
        let block_align = self.channels * (self.bits_per_sample / 8);
        let file_size = 36 + data_size; // Total file size minus 8 bytes for RIFF header

        let mut wav = Vec::with_capacity(44 + data_size as usize);

        // RIFF header
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        // fmt sub-chunk
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes()); // Sub-chunk size (16 for PCM)
        wav.extend_from_slice(&1u16.to_le_bytes()); // Audio format (1 = PCM)
        wav.extend_from_slice(&self.channels.to_le_bytes());
        wav.extend_from_slice(&self.sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_align.to_le_bytes());
        wav.extend_from_slice(&self.bits_per_sample.to_le_bytes());

        // data sub-chunk
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(&self.pcm_data);

        wav
    }
}

/// Initialize the I2S peripheral in standard RX mode for the INMP441 microphone.
///
/// Returns an I2S driver configured for 16 kHz, 32-bit data width (INMP441 native), mono.
/// The caller must keep the returned driver alive for the duration of recording.
pub fn init_i2s_microphone<'d>(
    i2s: impl Peripheral<P = impl esp_idf_hal::i2s::I2s> + 'd,
    bclk: impl Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin> + 'd,
    din: impl Peripheral<P = impl esp_idf_hal::gpio::InputPin> + 'd,
    ws: impl Peripheral<P = impl esp_idf_hal::gpio::InputPin + esp_idf_hal::gpio::OutputPin> + 'd,
) -> Result<I2sDriver<'d, I2sRx>, esp_idf_hal::sys::EspError> {
    let std_config = StdConfig::new(
        Config::default()
            .dma_buffer_count(config::I2S_DMA_BUF_COUNT)
            .frames_per_buffer(config::I2S_DMA_BUF_LEN),
        StdClkConfig::from_sample_rate_hz(config::VOICE_SAMPLE_RATE),
        StdSlotConfig::philips_slot_default(DataBitWidth::Bits32, SlotMode::Mono),
        StdGpioConfig::default(),
    );

    let driver = I2sDriver::<I2sRx>::new_std_rx(
        i2s,
        &std_config,
        bclk,
        din,
        None::<esp_idf_hal::gpio::AnyIOPin>, // No MCLK needed for INMP441
        ws,
    )?;

    log::info!("I2S microphone initialized: {}Hz, 32-bit, mono", config::VOICE_SAMPLE_RATE);
    Ok(driver)
}

/// Record audio from the I2S microphone into the provided buffer.
///
/// Reads 32-bit I2S frames from the INMP441 and converts to 16-bit PCM
/// by right-shifting 16 bits (the INMP441 outputs 24-bit data left-aligned
/// in a 32-bit frame; taking the upper 16 bits gives adequate resolution).
///
/// Returns the number of samples recorded, or an error.
pub fn record_chunk(
    driver: &mut I2sDriver<'_, I2sRx>,
    audio_buf: &mut AudioBuffer,
) -> Result<usize, esp_idf_hal::sys::EspError> {
    // Read buffer: I2S_DMA_BUF_LEN frames * 4 bytes per frame (32-bit mono)
    let mut i2s_buf = vec![0u8; config::I2S_DMA_BUF_LEN as usize * 4];

    let bytes_read = driver.read(&mut i2s_buf, 100)?;

    if bytes_read == 0 {
        return Ok(0);
    }

    let samples_read = bytes_read / 4; // 4 bytes per 32-bit sample

    // Convert 32-bit I2S samples to 16-bit PCM
    // INMP441 outputs 24-bit data left-aligned in 32-bit frame (MSB first in I2S)
    // In little-endian memory: [b0, b1, b2, b3] where b3 is MSB
    // We want the upper 16 bits: (i32_sample >> 16) as i16
    for i in 0..samples_read {
        let offset = i * 4;
        if offset + 3 < i2s_buf.len() {
            let raw_sample = i32::from_le_bytes([
                i2s_buf[offset],
                i2s_buf[offset + 1],
                i2s_buf[offset + 2],
                i2s_buf[offset + 3],
            ]);

            // Right-shift by 16 to get 16-bit value from upper bits
            let pcm_sample = (raw_sample >> 16) as i16;
            audio_buf.pcm_data.extend_from_slice(&pcm_sample.to_le_bytes());
        }
    }

    Ok(samples_read)
}
