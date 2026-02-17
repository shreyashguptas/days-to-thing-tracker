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
D0   ●  │           │  ● 5V
D1   ●  │           │  ● GND
D2   ●  │           │  ● 3V3
D3   ●  │           │  ● D10 (MOSI) ── Display SDA
D4   ●  │           │  ● D9  (MISO) ── Display BL
D5   ●  │           │  ● D8  (SCK)  ── Display SCK
D6   ●  │           │  ● D7  (RX)
        └───────────┘
```

## Battery Power

The XIAO ESP32-C6 has built-in LiPo battery support with USB-C charging.

### Wiring: 3.7V LiPo Battery (10,000mAh recommended)

| Battery Wire | XIAO Pad | Location |
|-------------|----------|----------|
| + (Red) | BAT+ | Bottom of board, near D5 marking |
| - (Black) | BAT- | Bottom of board, near D8 marking |

### Battery Pads Location

```
         USB-C
     ┌───────────┐
     │           │
     │   (top)   │
     │           │
     └───────────┘
     ┌───────────┐
     │ [BAT+]    │  ← Solder red wire here (near D5 side)
     │           │
     │ [BAT-]    │  ← Solder black wire here (near D8 side)
     └───────────┘
      (bottom of board)
```

### Charging

- Plug USB-C to charge the LiPo automatically (built-in charger IC)
- Red LED flashes during charging, turns off when full
- Device runs from battery when USB-C is disconnected
- No GPIO pins used — battery connects to dedicated pads only

### Power Saving

The firmware uses ESP32-C6 light sleep mode when idle:

| State | Description | Current Draw |
|-------|-------------|-------------|
| Active (screen on) | CPU running, WiFi on, display backlight on | ~90 mA |
| Sleep (screen off) | Light sleep, WiFi off, backlight off | ~3.1 mA |

With a 10,000mAh LiPo at ~4 uses/day (~2 min each): **~4 months battery life**.

Wake from sleep is instant (sub-millisecond) via encoder button press (GPIO2 interrupt).

### Alternative: D-Cell Batteries

For disposable batteries, use a battery holder:

| Config | Voltage | Connect To | Duration |
|--------|---------|-----------|----------|
| 3x D alkaline (series) | 4.5V | XIAO 5V pin (+) and GND (-) | ~6 months |
| 4x AA alkaline (series) | 6.0V | XIAO 5V pin (+) and GND (-) | ~1 month |

## Notes

- **3.3V only**: Both the display and encoder use 3.3V logic. Do NOT connect to 5V.
- **SPI**: The display is write-only. The D9/MISO pin is repurposed as the backlight control since no SPI read is needed.
- **Pull-ups**: The KY-040 module has onboard 10K pull-ups. The firmware also enables internal pull-ups for reliability.
- **Encoder behavior**: CLK falls before DT = clockwise. DT falls before CLK = counter-clockwise. SW goes LOW when pressed.
- **Long press**: Short press < 500ms. Long press >= 500ms.
- **Backlight timeout**: Auto-off after 5 minutes of inactivity (configurable in `firmware/src/config.rs`).
- **Light sleep**: In Station mode, the device enters light sleep after screen timeout. WiFi is stopped, CPU sleeps at ~3.1mA. Encoder button press wakes instantly.
