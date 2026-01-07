#!/usr/bin/env python3
"""
Rotary Encoder Handler for Raspberry Pi Kiosk

Converts rotary encoder input to keyboard events:
- Clockwise rotation -> Down arrow (scroll down)
- Counter-clockwise rotation -> Up arrow (scroll up)
- Button press -> Enter key (select/click)

GPIO Pin Configuration:
- CLK (A): GPIO 17
- DT (B):  GPIO 27
- SW:      GPIO 22
"""

import subprocess
import sys
import time
from signal import pause

try:
    from gpiozero import RotaryEncoder, Button
except ImportError:
    print("Error: gpiozero not installed. Run: sudo apt install python3-gpiozero")
    sys.exit(1)

# GPIO Pin Configuration
PIN_CLK = 17  # Encoder CLK (A)
PIN_DT = 27   # Encoder DT (B)
PIN_SW = 22   # Encoder Switch (button)

# Debounce settings
ROTATION_DEBOUNCE = 0.05  # seconds
BUTTON_DEBOUNCE = 0.3     # seconds

# Track last action time for debouncing
last_rotation_time = 0
last_button_time = 0


def send_key(key: str) -> None:
    """Send a keyboard event using xdotool."""
    try:
        subprocess.run(
            ["xdotool", "key", key],
            env={"DISPLAY": ":0"},
            check=True,
            capture_output=True
        )
    except subprocess.CalledProcessError as e:
        print(f"Error sending key {key}: {e}")
    except FileNotFoundError:
        print("Error: xdotool not installed. Run: sudo apt install xdotool")


def on_rotate_clockwise() -> None:
    """Handle clockwise rotation - scroll down."""
    global last_rotation_time
    now = time.time()
    if now - last_rotation_time > ROTATION_DEBOUNCE:
        last_rotation_time = now
        print("Clockwise -> Down")
        send_key("Down")


def on_rotate_counter_clockwise() -> None:
    """Handle counter-clockwise rotation - scroll up."""
    global last_rotation_time
    now = time.time()
    if now - last_rotation_time > ROTATION_DEBOUNCE:
        last_rotation_time = now
        print("Counter-clockwise -> Up")
        send_key("Up")


def on_button_press() -> None:
    """Handle button press - Enter/select."""
    global last_button_time
    now = time.time()
    if now - last_button_time > BUTTON_DEBOUNCE:
        last_button_time = now
        print("Button -> Enter")
        send_key("Return")


def main() -> None:
    """Main function to set up encoder and button handlers."""
    print("Rotary Encoder Handler Starting...")
    print(f"  CLK: GPIO {PIN_CLK}")
    print(f"  DT:  GPIO {PIN_DT}")
    print(f"  SW:  GPIO {PIN_SW}")
    print("")
    print("Controls:")
    print("  Clockwise     -> Down arrow (scroll down)")
    print("  Counter-clock -> Up arrow (scroll up)")
    print("  Button press  -> Enter (select)")
    print("")
    print("Press Ctrl+C to exit")
    print("")

    # Set up rotary encoder
    # Note: gpiozero's RotaryEncoder handles the quadrature decoding
    encoder = RotaryEncoder(PIN_CLK, PIN_DT, wrap=False, max_steps=0)

    # Track position for direction detection
    last_steps = encoder.steps

    # Set up button with pull-up (button connects to GND when pressed)
    button = Button(PIN_SW, pull_up=True, bounce_time=0.1)
    button.when_pressed = on_button_press

    # Polling loop for encoder rotation
    # (gpiozero's when_rotated callbacks can be unreliable)
    try:
        while True:
            current_steps = encoder.steps
            if current_steps > last_steps:
                on_rotate_clockwise()
            elif current_steps < last_steps:
                on_rotate_counter_clockwise()
            last_steps = current_steps
            time.sleep(0.01)  # 10ms polling interval
    except KeyboardInterrupt:
        print("\nShutting down...")


if __name__ == "__main__":
    main()
