# GPIO Pinout

## Hardware

| Component | Model | Notes |
|-----------|-------|-------|
| Microcontroller | Seeed XIAO ESP32-C6 | RISC-V, WiFi 6, BLE 5 |
| Display | 1.8" 160x128 TFT (ST7735) | SPI interface, RGB565 |
| Encoder | KY-040 Rotary Encoder | With push button switch |

## XIAO ESP32-C6 Pin Map

| Board Label | GPIO | Function in this project |
|-------------|------|--------------------------|
| D0 | GPIO0 | Encoder CLK (A) |
| D1 | GPIO1 | Encoder DT (B) |
| D2 | GPIO2 | Encoder Switch |
| D3 | GPIO21 | Display CS (chip select) |
| D4 | GPIO22 | Display DC (data/command) |
| D5 | GPIO23 | Display RST (reset) |
| D6 | GPIO16 | *unused* (UART TX) |
| D7 | GPIO17 | *unused* (UART RX) |
| D8 (SCK) | GPIO19 | Display SCK (SPI clock) |
| D9 (MISO) | GPIO20 | Display Backlight |
| D10 (MOSI) | GPIO18 | Display MOSI (SPI data) |

### Reserved Pins (not exposed / do not use)

| GPIO | Function |
|------|----------|
| GPIO3 | RF switch power (must be LOW for antenna) |
| GPIO14 | RF antenna selector (LOW=built-in, HIGH=external) |
| GPIO15 | Onboard user LED |
| GPIO9 | Boot button |

## Wiring: TFT Display (ST7735 160x128)

| Display Pin | XIAO Pin | GPIO | Description |
|-------------|----------|------|-------------|
| SCK/SCL | D8 | GPIO19 | SPI clock |
| MOSI/SDA | D10 | GPIO18 | SPI data |
| CS | D3 | GPIO21 | Chip select |
| DC/RS | D4 | GPIO22 | Data/Command select |
| RST/RES | D5 | GPIO23 | Reset (active low) |
| BL | D9 | GPIO20 | Backlight (HIGH = on) |
| VCC | 3V3 | - | Power (3.3V) |
| GND | GND | - | Ground |

## Wiring: Rotary Encoder (KY-040)

| Encoder Pin | XIAO Pin | GPIO | Description |
|-------------|----------|------|-------------|
| CLK (A) | D0 | GPIO0 | Rotation signal |
| DT (B) | D1 | GPIO1 | Direction signal |
| SW | D2 | GPIO2 | Push button (active low) |
| + | 3V3 | - | Power (3.3V) |
| GND | GND | - | Ground |

## XIAO ESP32-C6 Board Layout

```
         USB-C
     ┌───────────┐
D0   │ o       o │  D10 (MOSI) ── Display SDA
D1   │ o       o │  D9  (MISO) ── Display BL
D2   │ o       o │  D8  (SCK)  ── Display SCK
D3   │ o       o │  D7  (RX)
D4   │ o       o │  D6  (TX)
D5   │ o       o │  3V3
GND  │ o       o │  GND
     └───────────┘
         [RST]
```

## Notes

- **3.3V only**: Both the display and encoder use 3.3V logic. Do NOT connect to 5V.
- **SPI**: The display is write-only. The D9/MISO pin is repurposed as the backlight control since no SPI read is needed.
- **Pull-ups**: The KY-040 module has onboard 10K pull-ups. The firmware also enables internal pull-ups for reliability.
- **Encoder behavior**: CLK falls before DT = clockwise. DT falls before CLK = counter-clockwise. SW goes LOW when pressed.
- **Long press**: Short press < 500ms. Long press >= 500ms.
- **Backlight timeout**: Auto-off after 5 minutes of inactivity (configurable in `firmware/src/config.rs`).
