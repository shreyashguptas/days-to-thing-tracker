#!/usr/bin/env python3
"""
Rotary Encoder Handler for Raspberry Pi Kiosk

Converts rotary encoder input to keyboard events:
- Clockwise rotation -> Down arrow (move focus down)
- Counter-clockwise rotation -> Up arrow (move focus up)
- Short button press -> Enter key (select/activate)
- Long button press (>0.5s) -> Escape key (back/cancel)

GPIO Pin Configuration:
- CLK (A): GPIO 17
- DT (B):  GPIO 27
- SW:      GPIO 22
- BL:      GPIO 18 (display backlight)
"""

import subprocess
import sys
import time
from signal import pause

try:
    from gpiozero import RotaryEncoder, Button, OutputDevice
except ImportError:
    print("Error: gpiozero not installed. Run: sudo apt install python3-gpiozero")
    sys.exit(1)

# GPIO Pin Configuration
PIN_CLK = 17       # Encoder CLK (A)
PIN_DT = 27        # Encoder DT (B)
PIN_SW = 22        # Encoder Switch (button)
PIN_BACKLIGHT = 18 # Display backlight

# Debounce settings
ROTATION_DEBOUNCE = 0.02  # 20ms - snappy rotation response
BUTTON_DEBOUNCE = 0.2     # seconds

# Long press threshold
LONG_PRESS_TIME = 0.5  # seconds - hold longer than this for Escape

# Idle timeout for screen power saving
IDLE_TIMEOUT = 300  # 5 minutes in seconds

# Track last action time for debouncing
last_rotation_time = 0
last_button_time = 0
button_press_time = 0  # Track when button was pressed

# Idle timeout state
last_activity_time = 0  # Will be initialized in main()
screen_is_on = True
backlight = None  # Will be initialized in main()


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


def screen_off() -> None:
    """Turn off the display backlight via GPIO."""
    global screen_is_on
    if screen_is_on and backlight is not None:
        backlight.off()
        screen_is_on = False
        print("Backlight OFF (idle timeout)")


def screen_on() -> None:
    """Turn on the display backlight via GPIO."""
    global screen_is_on
    if not screen_is_on and backlight is not None:
        backlight.on()
        screen_is_on = True
        print("Backlight ON (user activity)")


def record_activity() -> None:
    """Record user activity and wake screen immediately if off."""
    global last_activity_time, screen_is_on
    last_activity_time = time.time()
    # Inline wake-up for instant response (bypass screen_on() overhead)
    if not screen_is_on and backlight is not None:
        backlight.on()
        screen_is_on = True
        print("Backlight ON (user activity)")


def on_rotate_clockwise() -> None:
    """Handle clockwise rotation - scroll down."""
    global last_rotation_time
    now = time.time()
    if now - last_rotation_time > ROTATION_DEBOUNCE:
        last_rotation_time = now
        record_activity()
        print("Clockwise -> Down")
        send_key("Down")


def on_rotate_counter_clockwise() -> None:
    """Handle counter-clockwise rotation - scroll up."""
    global last_rotation_time
    now = time.time()
    if now - last_rotation_time > ROTATION_DEBOUNCE:
        last_rotation_time = now
        record_activity()
        print("Counter-clockwise -> Up")
        send_key("Up")


def on_button_pressed() -> None:
    """Handle button press start - record time."""
    global button_press_time
    record_activity()
    button_press_time = time.time()


def on_button_released() -> None:
    """Handle button release - determine short or long press."""
    global last_button_time, button_press_time
    now = time.time()

    # Debounce check
    if now - last_button_time < BUTTON_DEBOUNCE:
        return

    last_button_time = now
    press_duration = now - button_press_time

    if press_duration >= LONG_PRESS_TIME:
        # Long press -> Escape (back/cancel)
        print(f"Long press ({press_duration:.2f}s) -> Escape")
        send_key("Escape")
    else:
        # Short press -> Enter (select)
        print(f"Short press ({press_duration:.2f}s) -> Enter")
        send_key("Return")


def main() -> None:
    """Main function to set up encoder and button handlers."""
    global last_activity_time, backlight

    print("Rotary Encoder Handler Starting...")
    print(f"  CLK: GPIO {PIN_CLK}")
    print(f"  DT:  GPIO {PIN_DT}")
    print(f"  SW:  GPIO {PIN_SW}")
    print(f"  BL:  GPIO {PIN_BACKLIGHT}")
    print("")
    print("Controls:")
    print("  Clockwise      -> Down arrow (move focus down)")
    print("  Counter-clock  -> Up arrow (move focus up)")
    print("  Short press    -> Enter (select)")
    print(f"  Long press (>{LONG_PRESS_TIME}s) -> Escape (back)")
    print("")
    print(f"Backlight will turn off after {IDLE_TIMEOUT}s of inactivity")
    print("Press Ctrl+C to exit")
    print("")

    # Initialize backlight control (start with backlight ON)
    # Using OutputDevice for direct GPIO control (faster than LED)
    backlight = OutputDevice(PIN_BACKLIGHT, initial_value=True)

    # Initialize idle timeout tracking
    last_activity_time = time.time()

    # Set up rotary encoder
    # Note: gpiozero's RotaryEncoder handles the quadrature decoding
    encoder = RotaryEncoder(PIN_CLK, PIN_DT, wrap=False, max_steps=0)

    # Track position for direction detection
    last_steps = encoder.steps

    # Set up button with pull-up (button connects to GND when pressed)
    # Use both press and release to detect long press
    button = Button(PIN_SW, pull_up=True, bounce_time=0.05)
    button.when_pressed = on_button_pressed
    button.when_released = on_button_released

    # Polling loop for encoder rotation
    # (gpiozero's when_rotated callbacks can be unreliable)
    # Note: gpiozero counts 2 physical detents as 1 step, so we fire 2x per step
    try:
        while True:
            current_steps = encoder.steps
            diff = current_steps - last_steps
            if diff != 0:
                # Fire handler for each detent (2x per gpiozero step)
                num_inputs = abs(diff) * 2
                for _ in range(num_inputs):
                    if diff > 0:
                        on_rotate_clockwise()
                    else:
                        on_rotate_counter_clockwise()
                last_steps = current_steps

            # Check for idle timeout
            if screen_is_on and (time.time() - last_activity_time > IDLE_TIMEOUT):
                screen_off()

            time.sleep(0.01)  # 10ms polling interval
    except KeyboardInterrupt:
        print("\nShutting down...")


if __name__ == "__main__":
    main()
