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

import json
import subprocess
import sys
import threading
import time
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

try:
    from gpiozero import Button, DigitalOutputDevice
except ImportError:
    print("Error: gpiozero not installed. Run: sudo apt install python3-gpiozero")
    sys.exit(1)

# GPIO Pin Configuration
PIN_CLK = 17       # Encoder CLK (A)
PIN_DT = 27        # Encoder DT (B)
PIN_SW = 22        # Encoder Switch (button)
PIN_BACKLIGHT = 18 # Display backlight

# Debounce settings (only for button, not rotation)
BUTTON_DEBOUNCE = 0.2     # seconds

# Long press threshold
LONG_PRESS_TIME = 0.5  # seconds - hold longer than this for Escape

# Idle timeout for screen power saving
IDLE_TIMEOUT = 300  # 5 minutes in seconds

# Track last action time for debouncing
last_button_time = 0
button_press_time = 0  # Track when button was pressed

# Idle timeout state
last_activity_time = 0  # Will be initialized in main()
screen_is_on = True
backlight_device = None  # DigitalOutputDevice for GPIO backlight control

# Settings server configuration
SETTINGS_PORT = 8765
screen_timeout_enabled = True  # Can be toggled via HTTP API


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
    """Control backlight via GPIO. Returns True if successful."""
    if backlight_device is None:
        return False
    try:
        if on:
            backlight_device.on()
        else:
            backlight_device.off()
        return True
    except Exception as e:
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
    record_activity()
    print("Clockwise -> Down")
    send_key("Down")


def on_rotate_counter_clockwise() -> None:
    """Handle counter-clockwise rotation - scroll up."""
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


class SettingsHandler(BaseHTTPRequestHandler):
    """HTTP handler for settings API requests from the kiosk browser."""

    def do_OPTIONS(self):
        """Handle CORS preflight requests."""
        self.send_response(200)
        self._send_cors_headers()
        self.end_headers()

    def do_GET(self):
        """Return current settings."""
        if self.path == '/settings':
            self.send_response(200)
            self._send_cors_headers()
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            response = json.dumps({
                'screenTimeoutEnabled': screen_timeout_enabled
            })
            self.wfile.write(response.encode())
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        """Update settings."""
        global screen_timeout_enabled
        if self.path == '/settings':
            content_length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(content_length)
            try:
                data = json.loads(body)
                if 'screenTimeoutEnabled' in data:
                    screen_timeout_enabled = bool(data['screenTimeoutEnabled'])
                    print(f"Screen timeout {'enabled' if screen_timeout_enabled else 'disabled'}")

                self.send_response(200)
                self._send_cors_headers()
                self.send_header('Content-Type', 'application/json')
                self.end_headers()
                response = json.dumps({'success': True, 'screenTimeoutEnabled': screen_timeout_enabled})
                self.wfile.write(response.encode())
            except json.JSONDecodeError:
                self.send_response(400)
                self.end_headers()
        else:
            self.send_response(404)
            self.end_headers()

    def _send_cors_headers(self):
        """Send CORS headers for cross-origin access from kiosk."""
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')

    def log_message(self, format, *args):
        """Suppress HTTP request logging to keep console clean."""
        pass


def start_settings_server():
    """Start HTTP settings server in background thread."""
    server = HTTPServer(('127.0.0.1', SETTINGS_PORT), SettingsHandler)
    print(f"Settings server listening on localhost:{SETTINGS_PORT}")
    server.serve_forever()


def init_backlight_gpio() -> DigitalOutputDevice | None:
    """Initialize GPIO for backlight control. Returns device or None."""
    try:
        # Create output device for backlight pin, initially ON
        device = DigitalOutputDevice(PIN_BACKLIGHT, initial_value=True)
        print(f"  Backlight GPIO {PIN_BACKLIGHT} initialized (ON)")
        return device
    except Exception as e:
        print(f"  Failed to initialize backlight GPIO: {e}")
        return None


def main() -> None:
    """Main function to set up encoder and button handlers."""
    global last_activity_time, backlight_device

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

    # Initialize backlight control via GPIO
    backlight_device = init_backlight_gpio()
    if backlight_device:
        print(f"Backlight control: GPIO {PIN_BACKLIGHT} (direct)")
        print(f"Screen timeout: {IDLE_TIMEOUT}s of inactivity")
    else:
        print("Backlight control: not available (screen timeout disabled)")
    print("")
    print("Press Ctrl+C to exit")
    print("")

    # Initialize idle timeout tracking
    last_activity_time = time.time()

    # Start settings HTTP server in background thread
    settings_thread = threading.Thread(target=start_settings_server, daemon=True)
    settings_thread.start()

    # Set up encoder using raw edge detection (not RotaryEncoder)
    # This gives true 1:1 detent-to-event mapping
    clk = Button(PIN_CLK, pull_up=True)
    dt = Button(PIN_DT, pull_up=True)

    # Track CLK state for edge detection
    last_clk = clk.is_pressed

    # Set up button with pull-up (button connects to GND when pressed)
    button = Button(PIN_SW, pull_up=True, bounce_time=0.05)
    button.when_pressed = on_button_pressed
    button.when_released = on_button_released

    # Polling loop for encoder rotation
    # Detects falling edge on CLK (1 per detent) and reads DT for direction
    try:
        while True:
            clk_state = clk.is_pressed

            # Detect falling edge: CLK was HIGH (False), now LOW (True)
            # With pull_up=True: is_pressed=True means pin is LOW
            if clk_state and not last_clk:
                # At falling edge, check DT for direction
                # DT HIGH (is_pressed=False) = clockwise
                # DT LOW (is_pressed=True) = counter-clockwise
                if not dt.is_pressed:
                    on_rotate_clockwise()
                else:
                    on_rotate_counter_clockwise()

            last_clk = clk_state

            # Check for idle timeout (only if screen timeout is enabled)
            if screen_timeout_enabled and screen_is_on and (time.time() - last_activity_time > IDLE_TIMEOUT):
                screen_off()

            time.sleep(0.001)  # 1ms polling for responsive encoder
    except KeyboardInterrupt:
        print("\nShutting down...")


if __name__ == "__main__":
    main()
