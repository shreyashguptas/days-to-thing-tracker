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
- BL:      GPIO 18 (display backlight, controlled via sysfs)
"""

import subprocess
import sys
import time
from pathlib import Path

try:
    from gpiozero import Button
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
backlight_path = None  # Path to sysfs backlight control (if available)


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


def set_backlight(on: bool) -> bool:
    """Control backlight via sysfs. Returns True if successful."""
    if backlight_path is None:
        return False
    try:
        with open(backlight_path, 'w') as f:
            f.write('1' if on else '0')
        return True
    except (IOError, OSError) as e:
        print(f"Backlight control error: {e}")
        return False


def screen_off() -> None:
    """Turn off the display backlight."""
    global screen_is_on
    if screen_is_on:
        if set_backlight(False):
            screen_is_on = False
            print("Backlight OFF (idle timeout)")


def screen_on() -> None:
    """Turn on the display backlight."""
    global screen_is_on
    if not screen_is_on:
        if set_backlight(True):
            screen_is_on = True
            print("Backlight ON (user activity)")


def record_activity() -> None:
    """Record user activity and wake screen immediately if off."""
    global last_activity_time, screen_is_on
    last_activity_time = time.time()
    # Inline wake-up for instant response
    if not screen_is_on:
        if set_backlight(True):
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


def find_backlight_control() -> str | None:
    """Find sysfs path for backlight control. Returns path or None."""
    # Check for standard backlight interface
    backlight_dir = Path('/sys/class/backlight')
    if backlight_dir.exists():
        for bl in backlight_dir.iterdir():
            brightness_path = bl / 'brightness'
            if brightness_path.exists():
                print(f"  Found backlight: {bl.name}")
                return str(brightness_path)

    # Check for GPIO sysfs (if GPIO 18 is exported)
    gpio_path = Path(f'/sys/class/gpio/gpio{PIN_BACKLIGHT}/value')
    if gpio_path.exists():
        print(f"  Found GPIO {PIN_BACKLIGHT} sysfs")
        return str(gpio_path)

    return None


def main() -> None:
    """Main function to set up encoder and button handlers."""
    global last_activity_time, backlight_path

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

    # Initialize backlight control via sysfs
    backlight_path = find_backlight_control()
    if backlight_path:
        print(f"Backlight control: {backlight_path}")
        print(f"Backlight will turn off after {IDLE_TIMEOUT}s of inactivity")
    else:
        print("Backlight control: not available (screen timeout disabled)")
    print("")
    print("Press Ctrl+C to exit")
    print("")

    # Initialize idle timeout tracking
    last_activity_time = time.time()

    # Set up encoder using raw edge detection (not RotaryEncoder)
    # This gives true 1:1 detent-to-event mapping
    clk = Button(PIN_CLK, pull_up=True)
    dt = Button(PIN_DT, pull_up=True)

    # Track CLK state for edge detection
    last_clk = clk.is_pressed
    edge_count = 0  # Toggle to fire every other edge

    # Set up button with pull-up (button connects to GND when pressed)
    button = Button(PIN_SW, pull_up=True, bounce_time=0.05)
    button.when_pressed = on_button_pressed
    button.when_released = on_button_released

    # Polling loop for encoder rotation
    # Detects edges on CLK, fires every other edge for 1:1 detent mapping
    try:
        while True:
            clk_state = clk.is_pressed

            # Detect any edge on CLK (state change)
            if clk_state != last_clk:
                edge_count += 1
                last_clk = clk_state

                # Only fire on every other edge (encoder has 2 edges per detent)
                if edge_count % 2 == 0:
                    # Determine direction from DT state
                    if clk_state == dt.is_pressed:
                        on_rotate_clockwise()
                    else:
                        on_rotate_counter_clockwise()

            # Check for idle timeout
            if screen_is_on and (time.time() - last_activity_time > IDLE_TIMEOUT):
                screen_off()

            time.sleep(0.001)  # 1ms polling for responsive encoder
    except KeyboardInterrupt:
        print("\nShutting down...")


if __name__ == "__main__":
    main()
