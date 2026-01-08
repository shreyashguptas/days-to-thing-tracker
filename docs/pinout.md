# GPIO Pinout

## Raspberry Pi Zero 2 W Connections

### Rotary Encoder (KY-040)

| Encoder Pin | GPIO | Physical Pin | Description |
|------------|------|--------------|-------------|
| CLK (A)    | 17   | 11           | Clock signal |
| DT (B)     | 27   | 13           | Data signal |
| SW         | 22   | 15           | Push button |
| +          | -    | 1 (3.3V)     | Power |
| GND        | -    | 6            | Ground |

### TFT Display (ST7735 160x128)

| Display Pin | GPIO | Physical Pin | Description |
|------------|------|--------------|-------------|
| BL         | 18   | 12           | Backlight control |
| CS         | 8    | 24           | Chip select (SPI0 CE0) |
| DC/RS      | 25   | 22           | Data/Command |
| MOSI       | 10   | 19           | SPI data (SPI0 MOSI) |
| SCK        | 11   | 23           | SPI clock (SPI0 SCLK) |
| RST        | 24   | 18           | Reset |
| VCC        | -    | 1 (3.3V)     | Power |
| GND        | -    | 9            | Ground |

## Physical Pin Layout

```
                    3.3V (1)  (2)  5V
          (SDA1) GPIO 2 (3)  (4)  5V
         (SCL1) GPIO 3 (5)  (6)  GND
                GPIO 4 (7)  (8)  GPIO 14 (TXD)
                   GND (9)  (10) GPIO 15 (RXD)
      (CLK) --> GPIO 17 (11) (12) GPIO 18 <-- (BL)
       (DT) --> GPIO 27 (13) (14) GND
       (SW) --> GPIO 22 (15) (16) GPIO 23
                  3.3V (17) (18) GPIO 24 <-- (RST)
     (MOSI) --> GPIO 10 (19) (20) GND
               GPIO 9 (21) (22) GPIO 25 <-- (DC)
      (SCK) --> GPIO 11 (23) (24) GPIO 8 <-- (CS)
                   GND (25) (26) GPIO 7
              GPIO 0 (27) (28) GPIO 1
              GPIO 5 (29) (30) GND
              GPIO 6 (31) (32) GPIO 12
             GPIO 13 (33) (34) GND
             GPIO 19 (35) (36) GPIO 16
             GPIO 26 (37) (38) GPIO 20
                  GND (39) (40) GPIO 21
```

## Notes

1. **Pull-up resistors**: The encoder CLK, DT, and SW pins use internal pull-ups (configured in software).

2. **Backlight control**: GPIO 18 controls the display backlight. HIGH = on, LOW = off.

3. **SPI Configuration**: The display uses SPI0. Enable SPI in `raspi-config` or add `dtparam=spi=on` to `/boot/config.txt`.

4. **Framebuffer**: The display should appear as `/dev/fb0` or `/dev/fb1` when properly configured.

## Config.txt Additions

Add these lines to `/boot/config.txt`:

```
# Enable SPI
dtparam=spi=on

# ST7735 display overlay
dtoverlay=st7735r,dc_pin=25,reset_pin=24,led_pin=18,speed=32000000,width=160,height=128

# Rotate display if needed
display_rotate=0
```
