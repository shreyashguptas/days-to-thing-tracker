# Raspberry Pi Kiosk Display Documentation

This document covers the complete setup and architecture of the Raspberry Pi Zero 2 W kiosk display that shows the Days Tracker web application.

## Table of Contents

1. [Hardware Overview](#hardware-overview)
2. [GPIO Wiring](#gpio-wiring)
3. [Display Driver Configuration](#display-driver-configuration)
4. [X11 Framebuffer Configuration](#x11-framebuffer-configuration)
5. [Kiosk Browser Setup](#kiosk-browser-setup)
6. [Rotary Encoder Handler](#rotary-encoder-handler)
7. [Systemd Services](#systemd-services)
8. [Installation Process](#installation-process)
9. [Troubleshooting](#troubleshooting)

---

## Hardware Overview

### Components

| Component | Model | Specifications |
|-----------|-------|----------------|
| **Single Board Computer** | Raspberry Pi Zero 2 W | ARM Cortex-A53, 512MB RAM, WiFi |
| **Display** | 1.8" ST7735 TFT | 160x128 pixels, SPI interface, 65K colors |
| **Input Device** | KY-040 Rotary Encoder | Quadrature encoder with push button |

### Display Specifications

- **Resolution**: 160x128 pixels (landscape orientation)
- **Controller**: ST7735R
- **Interface**: SPI (Serial Peripheral Interface)
- **Color Depth**: 16-bit (65,536 colors)
- **Operating Voltage**: 3.3V logic, 5V backlight

### Rotary Encoder Specifications

- **Type**: Incremental quadrature encoder
- **Output**: 2-bit Gray code (CLK + DT)
- **Switch**: Momentary push button
- **Resolution**: 20 detents per revolution

---

## GPIO Wiring

### Display Connections (ST7735 TFT)

| TFT Pin | Function | Pi GPIO | Pi Physical Pin |
|---------|----------|---------|-----------------|
| VCC | Power | 3.3V | Pin 1 |
| GND | Ground | GND | Pin 6 |
| CS | Chip Select | CE0 (GPIO 8) | Pin 24 |
| RESET | Reset | GPIO 24 | Pin 18 |
| A0/DC | Data/Command | GPIO 25 | Pin 22 |
| SDA | SPI Data (MOSI) | GPIO 10 | Pin 19 |
| SCK | SPI Clock | GPIO 11 | Pin 23 |
| LED | Backlight | GPIO 18 | Pin 12 |

### Rotary Encoder Connections

| Encoder Pin | Function | Pi GPIO | Pi Physical Pin |
|-------------|----------|---------|-----------------|
| CLK (A) | Clock/Phase A | GPIO 17 | Pin 11 |
| DT (B) | Data/Phase B | GPIO 27 | Pin 13 |
| SW | Switch (Button) | GPIO 22 | Pin 15 |
| + | Power | 3.3V | Pin 1 |
| GND | Ground | GND | Pin 9 |

### Wiring Diagram (ASCII)

```
Raspberry Pi Zero 2 W                    ST7735 TFT Display
┌─────────────────────┐                  ┌─────────────────┐
│ 3.3V (Pin 1)  ──────┼──────────────────┼── VCC           │
│ GND (Pin 6)   ──────┼──────────────────┼── GND           │
│ GPIO 8/CE0    ──────┼──────────────────┼── CS            │
│ GPIO 24       ──────┼──────────────────┼── RESET         │
│ GPIO 25       ──────┼──────────────────┼── DC (A0)       │
│ GPIO 10/MOSI  ──────┼──────────────────┼── SDA           │
│ GPIO 11/SCLK  ──────┼──────────────────┼── SCK           │
│ GPIO 18       ──────┼──────────────────┼── LED           │
└─────────────────────┘                  └─────────────────┘

Raspberry Pi Zero 2 W                    KY-040 Rotary Encoder
┌─────────────────────┐                  ┌─────────────────┐
│ GPIO 17       ──────┼──────────────────┼── CLK           │
│ GPIO 27       ──────┼──────────────────┼── DT            │
│ GPIO 22       ──────┼──────────────────┼── SW            │
│ 3.3V (Pin 1)  ──────┼──────────────────┼── +             │
│ GND (Pin 9)   ──────┼──────────────────┼── GND           │
└─────────────────────┘                  └─────────────────┘
```

---

## Display Driver Configuration

### Overview

The ST7735 display uses the **fbtft** (framebuffer TFT) driver, which creates a framebuffer device (`/dev/fb1`) that can be used like a standard display.

### Boot Configuration

The display is configured via `/boot/firmware/config.txt`. Add these lines:

```ini
# Enable SPI interface
dtparam=spi=on

# ST7735 TFT Display (1.8" 160x128) using fbtft driver
# rotate=270 gives landscape orientation
dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18

# Backlight control (set GPIO 18 as output, drive high)
gpio=18=op,dh

# Force HDMI output for fbcp (creates virtual fb0)
hdmi_force_hotplug=1

# Custom resolution matching TFT (landscape 160x128)
hdmi_cvt=160 128 60 1 0 0 0
hdmi_group=2
hdmi_mode=87

# Frame buffer settings
framebuffer_width=160
framebuffer_height=128
disable_overscan=1
```

### Overlay Parameters Explained

| Parameter | Value | Description |
|-----------|-------|-------------|
| `rotate` | 270 | Display rotation in degrees (0, 90, 180, 270) |
| `speed` | 32000000 | SPI clock speed in Hz (32 MHz) |
| `dc_pin` | 25 | GPIO pin for Data/Command signal |
| `reset_pin` | 24 | GPIO pin for display reset |
| `led_pin` | 18 | GPIO pin for backlight control |

### Verifying Display Driver

After reboot, verify the driver loaded correctly:

```bash
# Check for framebuffer devices
ls -la /dev/fb*
# Expected: /dev/fb0 (HDMI/virtual) and /dev/fb1 (TFT)

# Check driver messages
dmesg | grep fbtft
# Should show driver initialization messages

# Test display with static noise
cat /dev/urandom > /dev/fb1
# Display should show random colored pixels
```

---

## X11 Framebuffer Configuration

### Overview

X11 is configured to render directly to the TFT framebuffer (`/dev/fb1`) instead of the default HDMI output.

### Configuration File

Create `/etc/X11/xorg.conf.d/99-fbdev.conf`:

```
Section "Device"
    Identifier  "TFT"
    Driver      "fbdev"
    Option      "fbdev" "/dev/fb1"
EndSection

Section "Screen"
    Identifier  "TFT Screen"
    Device      "TFT"
EndSection
```

### Why This Configuration

- **Direct Rendering**: X11 renders directly to fb1, eliminating the need for fbcp (framebuffer copy)
- **Lower Latency**: No intermediate copy step means faster screen updates
- **Lower CPU Usage**: fbcp consumes CPU for continuous framebuffer copying

### Alternative: Using fbcp

If direct X11 rendering doesn't work, use fbcp to mirror fb0 to fb1:

```bash
# Build fbcp from source
git clone https://github.com/tasanakorn/rpi-fbcp.git
cd rpi-fbcp && mkdir build && cd build
cmake .. && make
sudo install fbcp /usr/local/bin/

# Run fbcp (mirrors fb0 → fb1)
/usr/local/bin/fbcp &
```

The kiosk.sh script automatically detects if fbcp is needed.

---

## Kiosk Browser Setup

### Kiosk Script (`kiosk.sh`)

Location: `/home/shreyash/kiosk.sh`

```bash
#!/bin/bash
# Raspberry Pi Kiosk Startup Script
# Starts X server and launches Chromium in kiosk mode

set -e

# Skip Chromium low-memory warning (Pi Zero 2 W has 512MB RAM)
export SKIP_MEMCHECK=1

# Configuration
KIOSK_URL="${KIOSK_URL:-https://days-tracker-server-deployment.reverse-python.ts.net/?kiosk=true}"
DISPLAY_WIDTH=160
DISPLAY_HEIGHT=128

# Detect Chromium binary (varies by Debian version)
detect_chromium_binary() {
    if command -v chromium &> /dev/null; then
        echo "chromium"         # Debian 12+
    elif command -v chromium-browser &> /dev/null; then
        echo "chromium-browser" # Older Debian
    else
        echo ""
    fi
}

CHROMIUM_BIN=$(detect_chromium_binary)

# Start fbcp if TFT is secondary framebuffer
if [[ -e /dev/fb1 ]] && command -v fbcp &> /dev/null; then
    /usr/local/bin/fbcp &
    sleep 1
fi

# Disable screen blanking and power management
xset s off          # Disable screen saver
xset -dpms          # Disable DPMS
xset s noblank      # Don't blank screen

# Hide mouse cursor
unclutter -idle 0.1 -root &

# Wait for network (Tailscale connectivity)
for i in {1..30}; do
    if curl -s --max-time 2 "$KIOSK_URL" > /dev/null 2>&1; then
        break
    fi
    sleep 2
done

# Launch Chromium in kiosk mode
exec "$CHROMIUM_BIN" \
    --no-memcheck \
    --kiosk \
    --noerrdialogs \
    --disable-infobars \
    --disable-session-crashed-bubble \
    --disable-restore-session-state \
    --disable-translate \
    --no-first-run \
    --fast \
    --fast-start \
    --disable-features=TranslateUI \
    --disk-cache-dir=/dev/null \
    --window-size=${DISPLAY_WIDTH},${DISPLAY_HEIGHT} \
    --window-position=0,0 \
    --check-for-update-interval=31536000 \
    "$KIOSK_URL"
```

### Chromium Flags Explained

| Flag | Purpose |
|------|---------|
| `--no-memcheck` | Skip low-memory warning (required for Pi Zero) |
| `--kiosk` | Full-screen kiosk mode, no browser UI |
| `--noerrdialogs` | Suppress error dialogs |
| `--disable-infobars` | Hide information bars |
| `--disable-session-crashed-bubble` | Don't show crash recovery dialog |
| `--disable-restore-session-state` | Don't restore previous tabs |
| `--no-first-run` | Skip first-run wizard |
| `--fast`, `--fast-start` | Optimize startup speed |
| `--disk-cache-dir=/dev/null` | Disable disk cache (preserve SD card) |
| `--window-size=160,128` | Match display resolution |
| `--check-for-update-interval=31536000` | Check updates once per year |

### Kiosk URL Parameter

The `?kiosk=true` URL parameter triggers the kiosk-optimized UI:
- Single task view (one task at a time)
- Large fonts optimized for 160x128
- Keyboard navigation (no mouse/touch)
- Simplified action menu

---

## Rotary Encoder Handler

### Overview

The encoder handler (`encoder.py`) converts physical encoder movements to keyboard events that control the kiosk browser.

### Input Mapping

| Physical Input | Duration | Keyboard Event | UI Action |
|---------------|----------|----------------|-----------|
| Clockwise rotation | - | Down Arrow | Move focus down / Next task |
| Counter-clockwise rotation | - | Up Arrow | Move focus up / Previous task |
| Button press | < 0.5s | Enter | Select / Confirm |
| Button press | ≥ 0.5s | Escape | Back / Cancel |

### Encoder Script (`encoder.py`)

Location: `/home/shreyash/encoder.py`

```python
#!/usr/bin/env python3
"""
Rotary Encoder Handler for Raspberry Pi Kiosk

Converts rotary encoder input to keyboard events using xdotool.
"""

import subprocess
import time
from gpiozero import RotaryEncoder, Button

# GPIO Configuration
PIN_CLK = 17  # Encoder CLK (A)
PIN_DT = 27   # Encoder DT (B)
PIN_SW = 22   # Encoder Switch

# Timing Configuration
ROTATION_DEBOUNCE = 0.05  # 50ms debounce for rotation
BUTTON_DEBOUNCE = 0.2     # 200ms debounce for button
LONG_PRESS_TIME = 0.5     # Hold > 0.5s for Escape

def send_key(key: str) -> None:
    """Send keyboard event via xdotool."""
    subprocess.run(
        ["xdotool", "key", key],
        env={"DISPLAY": ":0"},
        check=True,
        capture_output=True
    )

# ... (full implementation in raspberry-pi-deployment/encoder.py)
```

### Key Dependencies

```bash
# Required packages
sudo apt install python3-gpiozero python3-lgpio xdotool
```

### How xdotool Works

`xdotool` sends synthetic keyboard events to X11:
- Events are sent to the active window (Chromium in kiosk mode)
- The kiosk UI listens for keyboard events to navigate

---

## Systemd Services

### Kiosk Service (`kiosk.service`)

Location: `/etc/systemd/system/kiosk.service`

```ini
[Unit]
Description=Days Tracker Kiosk Browser
After=network-online.target graphical.target
Wants=network-online.target

[Service]
Type=simple
User=shreyash
Group=shreyash

# Environment
Environment=DISPLAY=:0
Environment=XAUTHORITY=/home/shreyash/.Xauthority
Environment=KIOSK_URL=https://days-tracker-server-deployment.reverse-python.ts.net/?kiosk=true

WorkingDirectory=/home/shreyash

# Start X server with kiosk script
ExecStartPre=/bin/sleep 5
ExecStart=/usr/bin/xinit /home/shreyash/kiosk.sh -- :0 -nocursor

Restart=on-failure
RestartSec=10

# Memory limit for Pi Zero 2 W (512MB total)
MemoryMax=400M

[Install]
WantedBy=multi-user.target
```

### Encoder Service (`encoder.service`)

Location: `/etc/systemd/system/encoder.service`

```ini
[Unit]
Description=Rotary Encoder Input Handler
After=kiosk.service
Requires=kiosk.service

[Service]
Type=simple
User=shreyash
Environment=DISPLAY=:0
ExecStartPre=/bin/sleep 3
ExecStart=/usr/bin/python3 /home/shreyash/encoder.py

Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### Service Management Commands

```bash
# Enable services to start on boot
sudo systemctl enable kiosk.service encoder.service

# Start services now
sudo systemctl start kiosk.service encoder.service

# Check service status
sudo systemctl status kiosk.service encoder.service

# View service logs
journalctl -u kiosk.service -f
journalctl -u encoder.service -f

# Restart services after changes
sudo systemctl restart kiosk.service encoder.service
```

---

## Installation Process

### Prerequisites

1. Raspberry Pi Zero 2 W with Raspberry Pi OS (Debian 12/13)
2. SSH access to the Pi
3. Hardware connected (display + encoder)
4. Pi connected to same Tailscale network as server

### Step-by-Step Installation

#### 1. Install System Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install X server and utilities
sudo apt install -y \
  xserver-xorg \
  xinit \
  x11-xserver-utils \
  unclutter \
  xdotool

# Install Chromium browser
sudo apt install -y chromium

# Install Python GPIO libraries
sudo apt install -y python3-gpiozero python3-lgpio
```

#### 2. Configure Display Driver

```bash
# Edit boot config
sudo nano /boot/firmware/config.txt

# Add the display configuration (see Display Driver Configuration section)
```

#### 3. Configure X11 Framebuffer

```bash
# Create X11 config directory
sudo mkdir -p /etc/X11/xorg.conf.d

# Create framebuffer config
sudo nano /etc/X11/xorg.conf.d/99-fbdev.conf
# Add the fbdev configuration (see X11 section)
```

#### 4. Copy Kiosk Files

```bash
# From your development machine
scp raspberry-pi-deployment/kiosk.sh shreyash@pi-zero-2-w-1:~/
scp raspberry-pi-deployment/encoder.py shreyash@pi-zero-2-w-1:~/

# On the Pi, make kiosk script executable
chmod +x ~/kiosk.sh
```

#### 5. Install Systemd Services

```bash
# Copy service files
sudo cp kiosk.service /etc/systemd/system/
sudo cp encoder.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable services
sudo systemctl enable kiosk.service encoder.service
```

#### 6. Reboot and Verify

```bash
sudo reboot

# After reboot, check services
sudo systemctl status kiosk.service
sudo systemctl status encoder.service
```

---

## Troubleshooting

### Display Issues

#### Display is white/blank

```bash
# Check if fbtft driver loaded
dmesg | grep fbtft

# Check framebuffer devices
ls -la /dev/fb*

# If /dev/fb1 missing, overlay may be wrong
# Try different overlay names:
# dtoverlay=adafruit18,...  (most common)
# dtoverlay=st7735r,...     (alternative)
```

#### Colors look wrong (BGR swap)

Add `bgr=1` to the overlay parameters:
```ini
dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18,bgr=1
```

#### Display is rotated wrong

Change `rotate=` value (0, 90, 180, 270) until orientation is correct.

### Kiosk Issues

#### Chromium won't start

```bash
# Check for errors
journalctl -u kiosk.service -n 50

# Common issues:
# - Chromium binary name (chromium vs chromium-browser)
# - Missing DISPLAY environment variable
# - X server not starting
```

#### Website not loading

```bash
# Test network connectivity
ping google.com

# Test Tailscale
tailscale status

# Test URL directly
curl -I https://days-tracker-server-deployment.reverse-python.ts.net/
```

#### Low memory warnings

The `--no-memcheck` flag should suppress this. If not:
```bash
# Increase swap
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile
# Set CONF_SWAPSIZE=512
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

### Encoder Issues

#### Encoder not responding

```bash
# Test encoder manually
python3 encoder.py
# Rotate encoder - should see "Clockwise -> Down" or "Counter-clockwise -> Up"
# Press button - should see "Short press -> Enter" or "Long press -> Escape"
```

#### xdotool not sending keys

```bash
# Check DISPLAY is set
echo $DISPLAY  # Should be :0

# Test xdotool manually
DISPLAY=:0 xdotool key Down
```

#### Encoder is jittery

Increase debounce times in `encoder.py`:
```python
ROTATION_DEBOUNCE = 0.1   # Increase from 0.05
BUTTON_DEBOUNCE = 0.3     # Increase from 0.2
```

### Service Issues

#### Service fails to start

```bash
# Check detailed status
sudo systemctl status kiosk.service -l

# Check full logs
journalctl -u kiosk.service --no-pager

# Common issues:
# - File not found (check paths)
# - Permission denied (check User= in service)
# - Dependencies not met (check After= and Requires=)
```

#### Service starts but Chromium crashes

```bash
# Run kiosk script manually to see errors
DISPLAY=:0 ./kiosk.sh

# Check X server is running
ps aux | grep X
```

---

## Architecture Summary

```
┌─────────────────────────────────────────────────────────────────┐
│                     Raspberry Pi Zero 2 W                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐   │
│   │   encoder    │     │   kiosk      │     │   fbtft      │   │
│   │   .service   │     │   .service   │     │   driver     │   │
│   └──────┬───────┘     └──────┬───────┘     └──────┬───────┘   │
│          │                    │                    │            │
│          ▼                    ▼                    ▼            │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐   │
│   │  encoder.py  │     │  kiosk.sh    │     │  /dev/fb1    │   │
│   │  (gpiozero)  │     │  (xinit)     │     │  TFT display │   │
│   └──────┬───────┘     └──────┬───────┘     └──────────────┘   │
│          │                    │                    ▲            │
│          │                    ▼                    │            │
│          │             ┌──────────────┐            │            │
│          │             │  X11 Server  │────────────┘            │
│          │             │  (Xorg)      │                         │
│          │             └──────┬───────┘                         │
│          │                    │                                 │
│          │                    ▼                                 │
│          │             ┌──────────────┐                         │
│          └────────────▶│  Chromium    │                         │
│           (xdotool)    │  Kiosk Mode  │                         │
│                        └──────┬───────┘                         │
│                               │                                 │
└───────────────────────────────┼─────────────────────────────────┘
                                │
                                │ HTTPS (Tailscale)
                                ▼
                    ┌───────────────────────┐
                    │  Days Tracker Server  │
                    │  (Docker/Tailscale)   │
                    └───────────────────────┘
```

### Data Flow

1. **User Input**: User rotates encoder or presses button
2. **GPIO Detection**: `encoder.py` detects GPIO state changes via gpiozero
3. **Key Translation**: Movement translated to keyboard events (Up/Down/Enter/Escape)
4. **xdotool**: Synthetic key events sent to X11
5. **Browser**: Chromium receives key events, updates React UI state
6. **API Calls**: UI changes trigger API calls to server (via Tailscale)
7. **Display Update**: React re-renders, X11 draws to framebuffer
8. **Physical Display**: fbtft driver updates TFT display

---

## Configuration Reference

### Quick Reference Table

| Item | Value |
|------|-------|
| Pi Model | Raspberry Pi Zero 2 W |
| Display | ST7735 160x128 |
| Display Driver | fbtft (adafruit18 overlay) |
| SPI Speed | 32 MHz |
| Display Rotation | 270° (landscape) |
| Encoder CLK GPIO | 17 |
| Encoder DT GPIO | 27 |
| Encoder SW GPIO | 22 |
| Display DC GPIO | 25 |
| Display RST GPIO | 24 |
| Backlight GPIO | 18 |
| Pi Username | shreyash |
| Kiosk URL | `https://days-tracker-server-deployment.reverse-python.ts.net/?kiosk=true` |
| Memory Limit | 400MB (of 512MB) |

### File Locations

| File | Location on Pi |
|------|----------------|
| Kiosk script | `/home/shreyash/kiosk.sh` |
| Encoder script | `/home/shreyash/encoder.py` |
| Kiosk service | `/etc/systemd/system/kiosk.service` |
| Encoder service | `/etc/systemd/system/encoder.service` |
| Boot config | `/boot/firmware/config.txt` |
| X11 config | `/etc/X11/xorg.conf.d/99-fbdev.conf` |
