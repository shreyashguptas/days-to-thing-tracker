/// Configuration for Days Tracker Kiosk (ESP32-C6)

// GPIO pins (XIAO ESP32-C6 pinout)
// See docs/pinout.md for wiring diagram
//
// Display (ST7735 SPI):
pub const PIN_SCK: i32 = 19;      // D8  - SPI clock
pub const PIN_MOSI: i32 = 18;     // D10 - SPI MOSI
pub const PIN_CS: i32 = 21;       // D3  - Display chip select
pub const PIN_DC: i32 = 22;       // D4  - Display data/command
pub const PIN_RST: i32 = 23;      // D5  - Display reset (repurposed for I2S SD when voice enabled)
pub const PIN_BL: i32 = 20;       // D9  - Display backlight (MISO pin, unused for SPI read)
//
// Rotary Encoder (KY-040):
pub const PIN_ENC_CLK: i32 = 0;   // D0  - Encoder CLK (A)
pub const PIN_ENC_DT: i32 = 1;    // D1  - Encoder DT (B)
pub const PIN_ENC_SW: i32 = 2;    // D2  - Encoder switch
//
// INMP441 I2S Microphone:
// SCK and WS use the two remaining exposed pins (D6, D7).
// SD reuses the display RST pin (D5/GPIO23) — tie display RST HIGH via 10K pull-up.
pub const PIN_I2S_SCK: i32 = 16;  // D6  - I2S bit clock (BCLK)
pub const PIN_I2S_WS: i32 = 17;   // D7  - I2S word select (LRCLK)
pub const PIN_I2S_SD: i32 = 23;   // D5  - I2S serial data in (shared with display RST)

// Display settings
pub const DISPLAY_WIDTH: u32 = 160;
pub const DISPLAY_HEIGHT: u32 = 128;

// SoftAP WiFi settings
pub const AP_SSID: &str = "DaysTracker";
pub const AP_PASSWORD: &str = "tracker123";
pub const AP_MAX_CONNECTIONS: u16 = 4;
pub const AP_IP: &str = "192.168.4.1";

// HTTP server
pub const HTTP_PORT: u16 = 80;

// Timing
pub const POLL_INTERVAL_MS: u64 = 1;
pub const IDLE_TIMEOUT_SECS: u64 = 300;
pub const COMPLETING_DURATION_MS: u64 = 500;

// Storage
pub const STORAGE_PARTITION: &str = "storage";
pub const TASKS_FILE: &str = "/storage/tasks.json";
pub const HISTORY_FILE: &str = "/storage/history.json";

// NVS (Non-Volatile Storage) for WiFi credentials
pub const NVS_NAMESPACE: &str = "wifi";
pub const NVS_KEY_SSID: &str = "ssid";
pub const NVS_KEY_PASSWORD: &str = "password";

// SPI clock speed
pub const SPI_FREQ_HZ: u32 = 32_000_000;

// Voice control
pub const VOICE_SERVER_URL: &str = "http://192.168.1.100:8000/voice";  // Local dev server
pub const VOICE_MAX_RECORD_SECS: u32 = 5;
pub const VOICE_SAMPLE_RATE: u32 = 16_000;
pub const VOICE_TRIGGER_MS: u64 = 1000;  // Hold encoder > 1s to trigger voice
pub const VOICE_RESULT_TIMEOUT_SECS: u64 = 5;  // Auto-dismiss voice result after 5s

// I2S audio settings
pub const I2S_DMA_BUF_COUNT: u32 = 8;
pub const I2S_DMA_BUF_LEN: u32 = 512;  // frames per buffer
