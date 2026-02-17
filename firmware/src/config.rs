/// Configuration for Days Tracker Kiosk (ESP32-C6)

// GPIO pins (XIAO ESP32-C6 pinout)
// See docs/pinout.md for wiring diagram
//
// Display (ST7735 SPI):
#[allow(dead_code)]
pub const PIN_SCK: i32 = 19;      // D8  - SPI clock
#[allow(dead_code)]
pub const PIN_MOSI: i32 = 18;     // D10 - SPI MOSI
#[allow(dead_code)]
pub const PIN_CS: i32 = 21;       // D3  - Display chip select
#[allow(dead_code)]
pub const PIN_DC: i32 = 22;       // D4  - Display data/command
#[allow(dead_code)]
pub const PIN_RST: i32 = 23;      // D5  - Display reset
#[allow(dead_code)]
pub const PIN_BL: i32 = 20;       // D9  - Display backlight (MISO pin, unused for SPI read)
//
// Rotary Encoder (KY-040):
#[allow(dead_code)]
pub const PIN_ENC_CLK: i32 = 0;   // D0  - Encoder CLK (A)
#[allow(dead_code)]
pub const PIN_ENC_DT: i32 = 1;    // D1  - Encoder DT (B)
pub const PIN_ENC_SW: i32 = 2;    // D2  - Encoder switch (also light sleep wake source)

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
pub const IDLE_TIMEOUT_SECS: u64 = 15; // TODO: increase for normal use after power testing
pub const QR_IDLE_TIMEOUT_SECS: u64 = 120; // 2 minutes for QR/web UI screens
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
