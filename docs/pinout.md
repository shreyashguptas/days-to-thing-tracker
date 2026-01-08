# GPIO Pinout

## Hardware Used

| Component | Model | Notes |
|-----------|-------|-------|
| Computer | Raspberry Pi Zero 2 W | 64-bit, WiFi enabled |
| Display | 1.8" 160x128 TFT (ST7735) | SPI interface, RGB565 |
| Encoder | KY-040 Rotary Encoder | With push button switch |

## Raspberry Pi Zero 2 W Connections

### Rotary Encoder (KY-040)

| Encoder Pin | GPIO | Physical Pin | Description |
|-------------|------|--------------|-------------|
| CLK (A) | 17 | 11 | Clock/rotation signal |
| DT (B) | 27 | 13 | Direction signal |
| SW | 22 | 15 | Push button (active low) |
| + | - | 1 (3.3V) | Power |
| GND | - | 6 | Ground |

### TFT Display (ST7735 160x128)

| Display Pin | GPIO | Physical Pin | Description |
|-------------|------|--------------|-------------|
| BL | 18 | 12 | Backlight control (HIGH=on) |
| CS | 8 | 24 | Chip select (SPI0 CE0) |
| DC/RS | 25 | 22 | Data/Command select |
| MOSI/SDA | 10 | 19 | SPI data (SPI0 MOSI) |
| SCK/SCL | 11 | 23 | SPI clock (SPI0 SCLK) |
| RST/RES | 24 | 18 | Reset (active low) |
| VCC | - | 1 (3.3V) | Power (3.3V) |
| GND | - | 9 | Ground |

**Note:** Some displays label pins differently:
- SDA = MOSI (data)
- SCL = SCK (clock)
- DC = RS (data/command)
- RES = RST (reset)

## Physical Pin Layout

```
                    3.3V (1)  (2)  5V
          (SDA1) GPIO 2 (3)  (4)  5V
         (SCL1) GPIO 3 (5)  (6)  GND  <-- Encoder GND
                GPIO 4 (7)  (8)  GPIO 14 (TXD)
  Display GND -->  GND (9)  (10) GPIO 15 (RXD)
 Encoder CLK --> GPIO 17 (11) (12) GPIO 18 <-- Display BL
  Encoder DT --> GPIO 27 (13) (14) GND
  Encoder SW --> GPIO 22 (15) (16) GPIO 23
                  3.3V (17) (18) GPIO 24 <-- Display RST
Display MOSI --> GPIO 10 (19) (20) GND
               GPIO 9 (21) (22) GPIO 25 <-- Display DC
 Display SCK --> GPIO 11 (23) (24) GPIO 8 <-- Display CS
                   GND (25) (26) GPIO 7
              GPIO 0 (27) (28) GPIO 1
              GPIO 5 (29) (30) GND
              GPIO 6 (31) (32) GPIO 12
             GPIO 13 (33) (34) GND
             GPIO 19 (35) (36) GPIO 16
             GPIO 26 (37) (38) GPIO 20
                  GND (39) (40) GPIO 21
```

## Display Driver Configuration

### Raspberry Pi OS Bookworm (2024+)

The ST7735 display uses the **fbtft** framebuffer driver. Add to `/boot/firmware/config.txt`:

```ini
# Enable SPI interface
dtparam=spi=on

# ST7735 160x128 TFT Display
# Using fbtft/adafruit18 driver (creates /dev/fb0)
# Parameters:
#   dc_pin=25    - Data/Command GPIO
#   reset_pin=24 - Reset GPIO
#   speed=32000000 - 32MHz SPI clock
#   rotate=90    - Landscape orientation (160x128)
dtoverlay=adafruit18,dc_pin=25,reset_pin=24,speed=32000000,rotate=90
```

**Important:** Reboot after changing config.txt!

### Verifying the Display

After reboot, verify the framebuffer:

```bash
# Check framebuffer device exists
ls -la /dev/fb0

# Check driver name
cat /sys/class/graphics/fb0/name
# Should output: fb_st7735r

# Check resolution
cat /sys/class/graphics/fb0/virtual_size
# Should output: 160,128

# Test display (fills with red)
dd if=/dev/zero bs=1 count=$((160*128*2)) | tr '\0' '\377' > /dev/fb0

# Test display (fills with blue)
dd if=/dev/zero bs=1 count=$((160*128*2)) | tr '\0' '\037' > /dev/fb0
```

### Display Format

- Resolution: 160x128 pixels (landscape with rotate=90)
- Color format: RGB565 (16 bits per pixel)
- Framebuffer size: 160 * 128 * 2 = 40,960 bytes

### Alternative Overlays (Not Recommended)

Other overlays exist but the **adafruit18** overlay with fbtft driver works best:

| Overlay | Driver | Status |
|---------|--------|--------|
| `adafruit18` | fbtft (fb_st7735r) | **Recommended** |
| `adafruit-st7735r` | DRM | May not work properly |
| `st7735r` | DRM | May not work properly |

The DRM-based overlays create `/dev/dri/card*` devices but may not render correctly.

## Encoder Configuration

### Hardware Pull-ups

The KY-040 encoder module typically has onboard 10K pull-up resistors. The software also enables internal pull-ups for reliability.

### Encoder Behavior

- **Clockwise rotation**: CLK leads DT (CLK falls before DT)
- **Counter-clockwise rotation**: DT leads CLK (DT falls before CLK)
- **Button press**: SW goes LOW when pressed (active low)

### Long Press Detection

- Short press: < 500ms
- Long press: >= 500ms

## Backlight Control

GPIO 18 controls the display backlight:
- HIGH (1): Backlight ON
- LOW (0): Backlight OFF

The kiosk software automatically turns off the backlight after 5 minutes of inactivity (configurable).

## Power Notes

1. **3.3V only**: Both the display and encoder use 3.3V logic. Do NOT connect to 5V.
2. **Current draw**: The display with backlight draws ~20-40mA. The encoder draws minimal current.
3. **Ground**: Ensure all grounds are connected (display and encoder to Pi ground pins).

## Troubleshooting

### Display Issues

| Symptom | Possible Cause | Solution |
|---------|----------------|----------|
| White/blank screen | Wrong overlay | Use `adafruit18` overlay |
| No /dev/fb0 | SPI not enabled | Add `dtparam=spi=on` |
| Wrong colors | Wrong driver | Verify using fbtft driver |
| Upside down | Wrong rotation | Use `rotate=90` or `rotate=270` |
| No backlight | GPIO 18 not set | Check software backlight control |

### Encoder Issues

| Symptom | Possible Cause | Solution |
|---------|----------------|----------|
| No response | Wrong GPIO pins | Verify CLK=17, DT=27, SW=22 |
| Skips steps | Debounce issue | Check for loose connections |
| Wrong direction | CLK/DT swapped | Swap CLK and DT wires |
| Button not working | SW not connected | Check SW to GPIO 22 |
