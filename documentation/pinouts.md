# Hardware Pinout Reference

## ST7735S TFT Display (1.8" 128x160)

| Display Pin | Pi GPIO | Pi Physical Pin | Description |
|-------------|---------|-----------------|-------------|
| BL | GPIO 18 | Pin 12 | Backlight (PWM capable) |
| CS | GPIO 8 (CE0) | Pin 24 | Chip Select |
| DC | GPIO 25 | Pin 22 | Data/Command |
| RES | GPIO 24 | Pin 18 | Reset |
| SDA | GPIO 10 (MOSI) | Pin 19 | SPI Data |
| SK | GPIO 11 (SCLK) | Pin 23 | SPI Clock |
| VCC | 3.3V | Pin 1 or 17 | Power |
| GND | GND | Pin 6, 9, 14, 20, 25, etc. | Ground |

## Rotary Encoder (KY-040 or similar)

| Encoder Pin | Pi GPIO | Pi Physical Pin | Description |
|-------------|---------|-----------------|-------------|
| CLK | GPIO 17 | Pin 11 | Rotation signal A |
| DT | GPIO 27 | Pin 13 | Rotation signal B |
| SW | GPIO 22 | Pin 15 | Push button |
| + | 3.3V | Pin 1 or 17 | Power |
| GND | GND | Pin 6, 9, 14, 20, 25, etc. | Ground |

## Notes

- **Backlight Control**: GPIO 18 is used by `encoder.py` to turn the display backlight on/off for power saving after 5 minutes of inactivity.
- **SPI**: The display uses SPI0 (MOSI on GPIO 10, SCLK on GPIO 11, CE0 on GPIO 8).
- **Pull-ups**: The rotary encoder button uses the internal pull-up resistor (configured in software).
