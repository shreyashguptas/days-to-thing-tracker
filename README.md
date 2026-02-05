# Days Tracker Kiosk

A standalone embedded device for tracking recurring tasks. Runs on a Seeed XIAO ESP32-C6 with a 160x128 TFT display and rotary encoder. No internet required -- the device creates its own WiFi hotspot. Scan the QR code on the display to manage tasks from your phone.

## Features

- **Standalone WiFi hotspot**: Device creates its own network, works anywhere with power
- **Phone-based management**: Scan QR code to connect and manage tasks via web UI
- **160x128 TFT display**: Dark theme, task cards, dashboard with urgency counts
- **Rotary encoder navigation**: Scroll through tasks, select actions, long press to go back
- **Recurring tasks**: Daily, weekly, monthly, yearly recurrence
- **Completion history**: Track when tasks were completed
- **Screen timeout**: Automatic backlight off after 5 minutes idle
- **No internet / no cloud**: All data stored locally on device flash

## Hardware

| Component | Model | Notes |
|-----------|-------|-------|
| Microcontroller | Seeed XIAO ESP32-C6 | RISC-V, WiFi 6, ~$5 |
| Display | 1.8" ST7735S SPI TFT LCD Display Module | Full Color 128RGB*160 Dot-matrix, 128RGB*160 Dots resolution, 3.3V, SPI interface, Drive IC: ST7735S|
| Input | KY-040 Rotary Encoder | With push button |
| Power | USB-C (from XIAO board) | |

## Wiring

See [docs/pinout.md](docs/pinout.md) for full wiring diagram.

**Quick reference:**

| Component | Pin | XIAO Pin | GPIO |
|-----------|-----|----------|------|
| Display SCK | SCK/SCL | D8 | GPIO19 |
| Display MOSI | MOSI/SDA | D10 | GPIO18 |
| Display CS | CS | D3 | GPIO21 |
| Display DC | DC/RS | D4 | GPIO22 |
| Display RST | RST/RES | D5 | GPIO23 |
| Display Backlight | BL | D9 | GPIO20 |
| Encoder CLK | CLK | D0 | GPIO0 |
| Encoder DT | DT | D1 | GPIO1 |
| Encoder SW | SW | D2 | GPIO2 |
| Display/Encoder VCC | - | 3V3 | - |
| Display/Encoder GND | - | GND | - |

## Building and Flashing

### Prerequisites

Install the ESP32 Rust toolchain:

```bash
# Install espup (ESP32 Rust toolchain manager)
cargo install espup
espup install

# Install flash tool and linker proxy
cargo install espflash ldproxy

# Load the ESP environment (add to your shell profile)
source $HOME/export-esp.sh
```

### Build and Flash

```bash
cd firmware
cargo run --release
```

This builds the firmware and flashes it to the connected XIAO ESP32-C6 via USB-C. The serial monitor will start automatically after flashing.

### Build Only (no flash)

```bash
cd firmware
cargo build --release
```

## Usage

### First Boot

1. Power the device via USB-C
2. The display shows "Starting..." then "Starting WiFi..."
3. If no tasks exist, a QR code appears for WiFi connection
4. Scan the QR code with your phone to connect to the "DaysTracker" WiFi network
5. Open `http://192.168.4.1` in your phone browser to manage tasks

### Controls

| Action | Result |
|--------|--------|
| Rotate clockwise | Scroll down / Next item |
| Rotate counter-clockwise | Scroll up / Previous item |
| Short press | Select / Confirm |
| Long press (>0.5s) | Back / Go to previous screen |

### Dashboard

The home screen shows urgency counts:
- **Overdue**: Tasks past their due date
- **Today**: Tasks due today
- **This Week**: Tasks due within 7 days
- **Total**: All tasks

Select a category to filter, or select "All Tasks" to see everything.

### Task Actions

Press on a task card to see:
- **Done**: Mark task complete (advances to next due date)
- **History**: View completion history
- **Delete**: Remove the task
- **Back**: Return to task list

### Settings

From the dashboard, select "Settings":
- **Manage Tasks**: Shows WiFi QR code for phone access
- **Screen Timeout**: Toggle auto-off after 5 minutes idle

### Adding Tasks

1. Connect your phone to the "DaysTracker" WiFi network (password: `tracker123`)
2. Open `http://192.168.4.1` in your browser
3. Use the web interface to create, edit, or delete tasks
4. The kiosk display updates on the next interaction

## API

The device runs an HTTP server on port 80 at `192.168.4.1`:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Web UI |
| GET | `/health` | Health check |
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/:id` | Get task |
| PUT | `/api/tasks/:id` | Update task |
| DELETE | `/api/tasks/:id` | Delete task |
| POST | `/api/tasks/:id/complete` | Mark complete |
| GET | `/api/tasks/:id/history` | Completion history |
| POST | `/api/time` | Sync time from phone |

### Example: Create a Task

```bash
# Connect to DaysTracker WiFi first, then:
curl -X POST http://192.168.4.1/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Water Plants",
    "recurrenceType": "daily",
    "recurrenceValue": 3,
    "nextDueDate": "2025-01-10"
  }'
```

## Project Structure

```
firmware/
  .cargo/config.toml      # RISC-V target, ESP-IDF linker
  Cargo.toml               # Dependencies
  build.rs                 # ESP-IDF build integration
  sdkconfig.defaults       # ESP-IDF config (WiFi, partitions)
  partitions.csv           # Flash layout (2MB app + 1.9MB storage)
  src/
    main.rs                # Entry point, event loop
    config.rs              # Pin assignments, WiFi credentials, timings
    models.rs              # Task, CompletionRecord, RecurrenceType
    storage.rs             # JSON CRUD on flash
    views.rs               # View state machine (9 states)
    renderer.rs            # All UI rendering (13+ views)
    encoder.rs             # KY-040 rotary encoder via GPIO
    display.rs             # ST7735 SPI display + framebuffer
    theme.rs               # RGB565 color constants
    fonts.rs               # 5x7 and 12x18 bitmap font data
    http_server.rs         # REST API (10 endpoints)
    wifi.rs                # WiFi SoftAP setup
  static/
    index.html             # Web UI (embedded at compile time)
docs/
  pinout.md                # Wiring diagram
product-site/              # Marketing website
```

## Configuration

Edit `firmware/src/config.rs` to change:

| Setting | Default | Description |
|---------|---------|-------------|
| `AP_SSID` | `DaysTracker` | WiFi network name |
| `AP_PASSWORD` | `tracker123` | WiFi password |
| `IDLE_TIMEOUT_SECS` | `300` | Seconds before backlight off |
| `SPI_FREQ_HZ` | `32000000` | SPI clock speed (32 MHz) |

## Troubleshooting

### Display shows nothing

1. Check wiring matches [docs/pinout.md](docs/pinout.md)
2. Verify SPI connections: SCK (D8), MOSI (D10), CS (D3), DC (D4), RST (D5)
3. Check backlight wire on D9
4. Check serial monitor output for errors: `cargo run --release`

### Encoder not responding

1. Verify wiring: CLK (D0), DT (D1), SW (D2)
2. Check 3.3V and GND connections
3. Check serial monitor for encoder events

### Phone can't connect to WiFi

1. Look for "DaysTracker" network on your phone
2. Password is `tracker123`
3. If not visible, check serial monitor -- WiFi AP should show "ready"

### Web UI not loading

1. Ensure phone is connected to DaysTracker WiFi
2. Open `http://192.168.4.1` (not https)
3. Disable mobile data on your phone (some phones prefer cellular)

### Build errors

```bash
# Make sure ESP toolchain is loaded
source $HOME/export-esp.sh

# Clean and rebuild
cd firmware
cargo clean
cargo run --release
```

## License

MIT
