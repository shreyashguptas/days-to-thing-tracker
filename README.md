# Days Tracker Kiosk

A standalone kiosk application for tracking recurring tasks on a Raspberry Pi Zero 2 W with a 160x128 TFT display and rotary encoder.

## Features

- **Standalone operation**: No server required, runs entirely on the Pi
- **Direct framebuffer rendering**: No browser overhead, instant response
- **Rotary encoder navigation**: Scroll through tasks, select actions
- **QR code for mobile management**: Scan to add/edit tasks from your phone
- **Screen timeout**: Automatic backlight control to save power
- **REST API**: Manage tasks from any device on your network
- **SQLite storage**: Local database with optional SMB backup

## Hardware Requirements

| Component | Model | Notes |
|-----------|-------|-------|
| Single Board Computer | Raspberry Pi Zero 2 W | Other Pi models should work |
| Display | 160x128 TFT (ST7735) | SPI interface, 1.8" typical |
| Input | KY-040 Rotary Encoder | With push button |
| Storage | microSD card (8GB+) | For OS and data |
| Power | 5V 2A USB power supply | |

## Wiring

See [docs/pinout.md](docs/pinout.md) for complete pinout diagram.

**Quick Reference:**

| Component | Pin | GPIO | Physical Pin |
|-----------|-----|------|--------------|
| Display BL (Backlight) | BL | 18 | 12 |
| Display CS | CS | 8 | 24 |
| Display DC | DC | 25 | 22 |
| Display RST | RST | 24 | 18 |
| Display MOSI | SDA | 10 | 19 |
| Display SCK | SCL | 11 | 23 |
| Display VCC | - | - | 1 (3.3V) |
| Display GND | - | - | 9 |
| Encoder CLK | CLK | 17 | 11 |
| Encoder DT | DT | 27 | 13 |
| Encoder SW | SW | 22 | 15 |
| Encoder + | - | - | 1 (3.3V) |
| Encoder GND | - | - | 6 |

---

## Complete Installation Guide

This guide is for **Raspberry Pi OS Bookworm** (released 2024+) which includes Python 3.11+/3.13.

### Step 1: Flash Raspberry Pi OS

1. Download [Raspberry Pi Imager](https://www.raspberrypi.com/software/)
2. Select **Raspberry Pi OS Lite (64-bit)** - Bookworm or newer
3. Click the gear icon to configure:
   - Set hostname (e.g., `daystracker`)
   - Enable SSH with password authentication
   - Set username and password (e.g., `shreyash`)
   - Configure WiFi (SSID and password)
   - Set locale/timezone
4. Flash to your microSD card
5. Insert card into Pi and power on

### Step 2: Connect via SSH

```bash
# Wait 1-2 minutes for first boot, then:
ssh youruser@daystracker.local
# Or use IP address: ssh youruser@192.168.x.x
```

### Step 3: Configure Display Driver

The ST7735 display requires the fbtft framebuffer driver.

```bash
# Edit boot config
sudo nano /boot/firmware/config.txt
```

Add these lines at the end:

```ini
# Enable SPI
dtparam=spi=on

# ST7735 160x128 TFT Display (fbtft driver)
# DC=GPIO25, RST=GPIO24, rotate 90 for landscape
dtoverlay=adafruit18,dc_pin=25,reset_pin=24,speed=32000000,rotate=90
```

Save and exit (Ctrl+X, Y, Enter).

**Disable Linux console on TFT** (prevents login screen from appearing):

```bash
# Edit kernel command line
sudo nano /boot/firmware/cmdline.txt
```

Add `fbcon=map:10` to the END of the single line (don't create a new line):

```
... rootwait fbcon=map:10
```

This maps the console to a non-existent framebuffer, so the TFT stays blank until the kiosk starts.

**Reboot to apply:**
```bash
sudo reboot
```

### Step 4: Verify Display

After reboot, verify the framebuffer device exists:

```bash
ssh youruser@daystracker.local

# Check for framebuffer
ls -la /dev/fb*
# Should show: /dev/fb0

# Test display (should show red screen)
dd if=/dev/zero bs=1 count=$((160*128*2)) | tr '\0' '\377' > /dev/fb0
```

### Step 5: Install Dependencies

```bash
# Update system
sudo apt-get update
sudo apt-get upgrade -y

# Install required packages
sudo apt-get install -y \
    python3-pip \
    python3-venv \
    python3-dev \
    build-essential \
    pkg-config \
    libffi-dev \
    git \
    curl
```

### Step 6: Install Rust

Rust is required to build the kiosk-core library.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

### Step 7: Clone and Setup Project

```bash
# Clone repository
cd ~
git clone https://github.com/shreyashguptas/days-to-thing-tracker.git
cd days-to-thing-tracker

# Create Python virtual environment
python3 -m venv venv
source venv/bin/activate

# Install Python dependencies
pip install --upgrade pip
pip install -r python/requirements.txt

# Install maturin (Rust-Python build tool)
pip install maturin
```

### Step 8: Build Rust Library

```bash
cd ~/days-to-thing-tracker/rust/kiosk-core

# Build and install (this takes 5-10 minutes on Pi Zero 2 W)
maturin build --release

# Install the wheel
pip install target/wheels/*.whl

# Verify it works
python -c "import kiosk_core; print('kiosk_core loaded successfully')"
```

### Step 9: Create Data Directory

```bash
cd ~/days-to-thing-tracker
mkdir -p data
```

### Step 10: Install Systemd Services

```bash
# Copy service files
sudo cp systemd/kiosk-tracker.service /etc/systemd/system/
sudo cp systemd/kiosk-api.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable services to start at boot
sudo systemctl enable kiosk-tracker
sudo systemctl enable kiosk-api

# Start services
sudo systemctl start kiosk-tracker
sudo systemctl start kiosk-api
```

### Step 11: Verify Everything Works

```bash
# Check kiosk status
sudo systemctl status kiosk-tracker

# Check API status
sudo systemctl status kiosk-api

# View logs if needed
journalctl -u kiosk-tracker -f
journalctl -u kiosk-api -f
```

The display should now show "No tasks" or your task list. The API is available at `http://daystracker.local:8080`.

---

## Usage

### Controls

| Action | Result |
|--------|--------|
| Rotate clockwise | Scroll down / Next item |
| Rotate counter-clockwise | Scroll up / Previous item |
| Short press | Select / Confirm |
| Long press (>0.5s) | Back / Open Settings |

### Adding Tasks via QR Code

1. From the task list, **long press** to open Settings
2. Select **Manage Tasks**
3. A QR code appears - scan it with your phone
4. Use the mobile web interface to add/edit/delete tasks
5. **Long press** to go back to Settings, then select **Back**

### Adding Tasks via API

```bash
curl -X POST http://daystracker.local:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Water Plants",
    "recurrenceType": "days",
    "recurrenceValue": 3,
    "nextDueDate": "2025-01-10"
  }'
```

### Settings Menu

Long press from task list to access:
- **Manage Tasks**: Shows QR code for mobile management
- **Screen Timeout**: Toggle auto-off after 5 minutes idle
- **Back**: Return to task list

---

## Updating / Deploying Changes

When you make changes to the code and push to GitHub, follow these steps on the Pi:

### Quick Update (Python changes only)

```bash
ssh youruser@daystracker.local
cd ~/days-to-thing-tracker

# Pull latest changes
git pull

# Restart services
sudo systemctl restart kiosk-tracker
sudo systemctl restart kiosk-api
```

### Full Update (Rust changes)

```bash
ssh youruser@daystracker.local
cd ~/days-to-thing-tracker

# Pull latest changes
git pull

# Activate virtual environment
source venv/bin/activate

# Rebuild Rust library
cd rust/kiosk-core
maturin build --release
pip install target/wheels/*.whl --force-reinstall

# Restart services
cd ~/days-to-thing-tracker
sudo systemctl restart kiosk-tracker
sudo systemctl restart kiosk-api
```

### Update Script

For convenience, create an update script:

```bash
# Create update script
cat > ~/update-kiosk.sh << 'EOF'
#!/bin/bash
set -e
cd ~/days-to-thing-tracker
git pull
source venv/bin/activate
cd rust/kiosk-core
maturin build --release
pip install target/wheels/*.whl --force-reinstall
cd ~/days-to-thing-tracker
sudo systemctl restart kiosk-tracker
sudo systemctl restart kiosk-api
echo "Update complete!"
EOF

chmod +x ~/update-kiosk.sh
```

Then just run `~/update-kiosk.sh` to update.

---

## Troubleshooting

### Login screen appears on TFT at boot

The Linux console is being displayed on the TFT. Fix by disabling fbcon:

```bash
# Edit kernel command line
sudo nano /boot/firmware/cmdline.txt

# Add to END of line (don't create new line):
fbcon=map:10

# Reboot
sudo reboot
```

### Kiosk takes too long to start

Update the service file to start earlier:

```bash
sudo cp ~/days-to-thing-tracker/systemd/kiosk-tracker.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl restart kiosk-tracker
```

### Display shows white/blank

1. **Check wiring** - especially DC (GPIO 25) and RST (GPIO 24)
2. **Verify overlay** - ensure `/boot/firmware/config.txt` has the adafruit18 overlay
3. **Check framebuffer**:
   ```bash
   ls /dev/fb0  # Should exist
   cat /sys/class/graphics/fb0/name  # Should show "fb_st7735r"
   ```
4. **Reboot** after config changes

### Encoder not responding

1. **Check wiring** - CLK=GPIO17, DT=GPIO27, SW=GPIO22
2. **Check service logs**: `journalctl -u kiosk-tracker -f`
3. **Test GPIO**:
   ```bash
   sudo apt-get install -y python3-gpiozero
   python3 -c "from gpiozero import Button; b = Button(22); print('Press encoder...'); b.wait_for_press(); print('Pressed!')"
   ```

### Python 3.13 / PyO3 errors

The Rust library uses PyO3 0.22 which supports Python 3.13. If you get import errors:

```bash
cd ~/days-to-thing-tracker/rust/kiosk-core
source ~/days-to-thing-tracker/venv/bin/activate
maturin build --release
pip install target/wheels/*.whl --force-reinstall
```

### Service won't start

```bash
# Check logs
journalctl -u kiosk-tracker -n 50

# Common fixes:
# 1. Verify paths in service file match your username
sudo nano /etc/systemd/system/kiosk-tracker.service

# 2. Reload after editing
sudo systemctl daemon-reload
sudo systemctl restart kiosk-tracker
```

### API not accessible

1. **Check if running**: `sudo systemctl status kiosk-api`
2. **Check firewall**: `sudo ufw status` (if enabled, allow port 8080)
3. **Test locally**: `curl http://localhost:8080/health`

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Python Application                    │
│  main.py - Event loop, business logic                   │
│  database.py - SQLite operations                        │
│  views.py - Navigation state machine                    │
│  api.py - REST API + Web UI for remote management       │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 Rust Core (kiosk_core)                  │
│  encoder.rs - GPIO input handling (<10µs latency)       │
│  display.rs - Direct framebuffer rendering              │
│  renderer.rs - UI widgets and layout                    │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌────────────┴────────────┐
              │                         │
         GPIO 17,27,22             /dev/fb0
       (Rotary Encoder)          (TFT Display)
```

## Project Structure

```
days-to-thing-tracker/
├── python/                 # Python application
│   ├── main.py            # Entry point
│   ├── database.py        # SQLite operations
│   ├── models.py          # Data models
│   ├── views.py           # Navigation state
│   ├── api.py             # REST API
│   ├── config.py          # Configuration
│   └── templates/         # Web UI
│       └── index.html     # Mobile task management
├── rust/                   # Rust core library
│   └── kiosk-core/
│       └── src/
│           ├── lib.rs     # PyO3 module
│           ├── encoder.rs # GPIO handling
│           ├── display.rs # Framebuffer
│           ├── renderer.rs # UI rendering
│           └── theme.rs   # Colors
├── systemd/               # Service files
├── docs/                  # Documentation
│   └── pinout.md         # Wiring diagram
├── setup.sh               # Installation script
└── Cargo.toml             # Rust workspace
```

## API Endpoints

The REST API runs on port 8080:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Web UI for task management |
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/:id` | Get task |
| PUT | `/api/tasks/:id` | Update task |
| DELETE | `/api/tasks/:id` | Delete task |
| POST | `/api/tasks/:id/complete` | Mark complete |
| GET | `/api/tasks/:id/history` | Get history |
| GET | `/health` | Health check |

## Configuration

Environment variables (set in systemd service or shell):

| Variable | Default | Description |
|----------|---------|-------------|
| `KIOSK_DATA_DIR` | `./data` | Database location |
| `KIOSK_WEB_URL` | auto-detect | URL shown in QR code |
| `KIOSK_HOSTNAME` | system hostname | Hostname for QR URL |
| `SMB_BACKUP_ENABLED` | `false` | Enable SMB backup |
| `SMB_SHARE_PATH` | - | SMB share path |

## License

MIT
