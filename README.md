# Days Tracker Kiosk

A standalone embedded device for tracking recurring tasks. Runs on a Seeed XIAO ESP32-C6 with a 160x128 TFT display and rotary encoder. Creates its own WiFi hotspot -- scan the QR code on the display to manage tasks from your phone.

## Features

- **Standalone WiFi hotspot**: Device creates its own network, works anywhere with power
- **WiFi provisioning**: Optionally connect device to your home WiFi for local network access
- **Captive portal**: Automatically opens web UI when phone connects to device WiFi
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
| Display | 1.8" ST7735S SPI TFT | 128x160, 3.3V, SPI, full color |
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

These instructions cover building and flashing from a **Raspberry Pi** (or any Linux machine). The same toolchain also works on macOS.

### 1. System Dependencies (Raspberry Pi / Linux)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libudev-dev libssl-dev curl git
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 3. Install ESP32 Toolchain

```bash
# ESP32 Rust toolchain manager
cargo install espup
espup install

# Flash tool and linker proxy
cargo install espflash ldproxy

# Load the ESP environment (add this line to ~/.bashrc so it runs on every shell)
echo 'source $HOME/export-esp.sh' >> ~/.bashrc
source $HOME/export-esp.sh
```

### 4. Clone and Configure

```bash
git clone <repo-url>
cd days-to-thing-tracker
```

The file `firmware/sdkconfig.defaults` contains an absolute path to the partition table. Update it to match your system:

```bash
sed -i "s|CONFIG_PARTITION_TABLE_CUSTOM_FILENAME=.*|CONFIG_PARTITION_TABLE_CUSTOM_FILENAME=\"$(pwd)/firmware/partitions.csv\"|" firmware/sdkconfig.defaults
```

### 5. Build and Flash

Connect the XIAO ESP32-C6 to the Raspberry Pi via USB-C, then:

```bash
cd firmware
cargo run --release
```

This builds the firmware and flashes it to the board. The serial monitor starts automatically after flashing.

If the board isn't detected, put it in **bootloader mode**: hold the **B** (boot) button, press **R** (reset), then release **B**.

On Linux, if you get a permission error accessing the USB serial port:

```bash
sudo usermod -a -G dialout $USER
# Log out and back in for the group change to take effect
```

### Build Only (no flash)

```bash
cd firmware
cargo build --release
```

### Clean Build

If you hit build errors after toolchain updates:

```bash
cd firmware
cargo clean
cargo run --release
```

## Usage

### First Boot (AP Mode)

1. Power the device via USB-C
2. The device starts in **Access Point mode**, creating a "DaysTracker" WiFi network
3. If no tasks exist, a QR code appears for WiFi connection
4. Scan the QR code with your phone (or connect manually to "DaysTracker", password: `tracker123`)
5. A captive portal should auto-open; if not, check the device screen for the IP address
6. Use the web UI to add tasks and optionally provision your home WiFi

### WiFi Provisioning (Optional)

From the web UI, you can connect the device to your home WiFi:

1. The web UI scans for available networks
2. Select your home WiFi and enter the password
3. The device saves credentials and restarts in **Station mode**
4. The device joins your home WiFi and is accessible at its assigned IP
5. If the connection fails, the device clears saved credentials and restarts back into AP mode

To reset WiFi: from Settings on the device, select "Reset WiFi" and confirm.

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
- **Manage Tasks**: Shows QR code for phone access
- **Screen Timeout**: Toggle auto-off after 5 minutes idle
- **Reset WiFi**: Clear saved WiFi credentials and restart into AP mode

## API

The device runs an HTTP server on port 80. The device screen shows the current IP address.

### Task Endpoints

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

### WiFi Provisioning Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/wifi/status` | Current WiFi mode and IP |
| GET | `/api/wifi/scan` | Scan for available networks |
| POST | `/api/wifi/connect` | Connect to a network |
| DELETE | `/api/wifi/credentials` | Clear saved credentials |

### Example: Create a Task

```bash
# Connect to DaysTracker WiFi first, then use the IP shown on the device screen:
curl -X POST http://<device-ip>/api/tasks \
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
  partitions.csv           # Flash layout (1.5MB app + 2.4MB storage)
  src/
    main.rs                # Entry point, WiFi mode branching, event loop
    config.rs              # Pin assignments, WiFi credentials, timings
    models.rs              # Task, CompletionRecord, RecurrenceType
    storage.rs             # JSON CRUD on flash with backup/atomic writes
    views.rs               # View state machine (10 states)
    renderer.rs            # All UI rendering (16+ views)
    encoder.rs             # KY-040 rotary encoder via GPIO
    display.rs             # ST7735 SPI display + framebuffer
    theme.rs               # RGB565 color constants
    fonts.rs               # 5x7 and 12x18 bitmap font data
    http_server.rs         # REST API + WiFi provisioning + captive portal
    wifi.rs                # Dual-mode WiFi (SoftAP + Station), NVS credentials
    dns.rs                 # Captive portal DNS server (AP mode)
  static/
    index.html             # Web UI with WiFi provisioning (embedded at compile time)
docs/
  pinout.md                # Wiring diagram
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
2. Open the IP address shown on the device screen (use http, not https)
3. Disable mobile data on your phone (some phones prefer cellular)

### USB not detected on Raspberry Pi

1. Try a different USB-C cable (some are charge-only)
2. Check `ls /dev/ttyACM*` or `ls /dev/ttyUSB*` for the serial device
3. Put the board in bootloader mode: hold B, press R, release B
4. Add your user to the dialout group: `sudo usermod -a -G dialout $USER`

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
