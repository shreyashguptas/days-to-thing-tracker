# Raspberry Pi Kiosk Deployment

Deploy the Days Tracker web app on a Raspberry Pi Zero 2 W with a 1.8" TFT display and rotary encoder controls.

## Hardware Setup

### Components
- Raspberry Pi Zero 2 W
- 1.8" TFT Display (128x160, ST7735-based, SPI)
- Rotary Encoder with push button

### Wiring

**Display to Pi:**
| Display Pin | Pi GPIO | Pi Physical Pin |
|-------------|---------|-----------------|
| BL | GPIO 18 | Pin 12 |
| CS | GPIO 8 (CE0) | Pin 24 |
| DC | GPIO 25 | Pin 22 |
| RES | GPIO 24 | Pin 18 |
| SDA | GPIO 10 (MOSI) | Pin 19 |
| SK | GPIO 11 (SCLK) | Pin 23 |
| VCC | 3.3V | Pin 1 or 17 |
| GND | Ground | Pin 6, 9, 14, 20, or 25 |

**Rotary Encoder to Pi:**
| Encoder Pin | Pi GPIO | Pi Physical Pin |
|-------------|---------|-----------------|
| CLK | GPIO 17 | Pin 11 |
| DT | GPIO 27 | Pin 13 |
| SW | GPIO 22 | Pin 15 |
| + | 3.3V | Pin 1 or 17 |
| GND | Ground | Pin 6, 9, 14, 20, or 25 |

## Quick Install

1. Copy this folder to your Pi:
   ```bash
   scp -r raspberry-pi-deployment/ shreyash@pi-zero-2-w-1:~/
   ```

2. SSH into your Pi and run the installer:
   ```bash
   ssh shreyash@pi-zero-2-w-1
   cd ~/raspberry-pi-deployment
   chmod +x install.sh
   ./install.sh
   ```

3. Reboot when prompted.

## Manual Installation

### Step 1: Fix Display Driver

Edit `/boot/firmware/config.txt`:

```bash
sudo nano /boot/firmware/config.txt
```

Remove (if present):
```ini
dtoverlay=st7735r,dc_pin=25,reset_pin=24,speed=32000000,width=128,height=160
gpio=18=op,dh
```

Add:
```ini
# ST7735 TFT Display
dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18

# Framebuffer for kiosk
hdmi_force_hotplug=1
hdmi_cvt=160 128 60 1 0 0 0
hdmi_group=2
hdmi_mode=87
framebuffer_width=160
framebuffer_height=128
```

Reboot: `sudo reboot`

### Step 2: Verify Display

```bash
# Check framebuffer exists
ls -la /dev/fb*
# Should show fb0 AND fb1

# Test display
cat /dev/urandom > /dev/fb1
# Should show static on TFT (Ctrl+C to stop)
```

### Step 3: Install Dependencies

```bash
sudo apt update
sudo apt install -y cmake git build-essential \
    xserver-xorg xinit x11-xserver-utils \
    chromium-browser unclutter xdotool \
    python3-pip python3-gpiozero python3-lgpio
```

### Step 4: Build fbcp

```bash
cd ~
git clone https://github.com/tasanakorn/rpi-fbcp.git
cd rpi-fbcp && mkdir build && cd build
cmake .. && make
sudo install fbcp /usr/local/bin/
```

### Step 5: Install Scripts

```bash
cp ~/raspberry-pi-deployment/kiosk.sh ~/kiosk.sh
cp ~/raspberry-pi-deployment/encoder.py ~/encoder.py
chmod +x ~/kiosk.sh ~/encoder.py
```

### Step 6: Install Services

```bash
sudo cp ~/raspberry-pi-deployment/kiosk.service /etc/systemd/system/
sudo cp ~/raspberry-pi-deployment/encoder.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable kiosk encoder
sudo reboot
```

## Controls

| Input | Action |
|-------|--------|
| Rotate clockwise | Scroll down |
| Rotate counter-clockwise | Scroll up |
| Press button | Enter/Select |

## Troubleshooting

### Display shows only backlight (white/blank)

The driver isn't loading. Check:
```bash
dmesg | grep fbtft
ls -la /dev/fb*
```

If no fb1, try adding `green` parameter for green-tab displays:
```ini
dtoverlay=adafruit18,green,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18
```

### Colors look wrong (red/blue swapped)

Add `bgr=1` to the overlay:
```ini
dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18,bgr=1
```

### Display is rotated wrong

Change `rotate=` value (0, 90, 180, or 270):
```ini
dtoverlay=adafruit18,rotate=0,...   # or 90, 180, 270
```

### X server running at wrong resolution (display shows cursor only)

X may default to HDMI instead of the TFT. Verify with:
```bash
DISPLAY=:0 xrandr
```

If it shows 1024x768 instead of 160x128, install the X11 config:
```bash
sudo cp ~/raspberry-pi-deployment/99-fbdev.conf /etc/X11/xorg.conf.d/
sudo systemctl restart kiosk
```

### Chromium shows "low RAM" warning dialog

Pi Zero 2 W has 512MB RAM which triggers this warning. The `--no-memcheck` flag in kiosk.sh should prevent it. If it still appears:
```bash
sudo mkdir -p /etc/chromium.d
echo 'export SKIP_MEMCHECK=1' | sudo tee /etc/chromium.d/99-kiosk
sudo systemctl restart kiosk
```

### Website not loading

Check Tailscale:
```bash
tailscale status
curl -I https://days-tracker-server-deployment.reverse-python.ts.net/
```

### Services not starting

Check logs:
```bash
journalctl -u kiosk -f
journalctl -u encoder -f
```

### Encoder not responding

Test manually:
```bash
DISPLAY=:0 python3 ~/encoder.py
```

Check GPIO permissions - user must be in `gpio` group:
```bash
sudo usermod -a -G gpio shreyash
```

## Files

| File | Description |
|------|-------------|
| `install.sh` | Automated installer |
| `kiosk.sh` | Starts X + Chromium kiosk |
| `encoder.py` | Rotary encoder handler |
| `kiosk.service` | Systemd service for kiosk |
| `encoder.service` | Systemd service for encoder |
| `99-fbdev.conf` | X11 config for TFT framebuffer |
| `config.txt.additions` | Boot config reference |

## Configuration

Edit `kiosk.sh` or the systemd service to change the URL:
```bash
KIOSK_URL=https://your-tailscale-hostname/
```
