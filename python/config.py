"""
Configuration for Days Tracker Kiosk
"""
import os
from pathlib import Path

# Base paths
BASE_DIR = Path(__file__).parent.parent
DATA_DIR = Path(os.environ.get("KIOSK_DATA_DIR", BASE_DIR / "data"))

# Ensure data directory exists
DATA_DIR.mkdir(parents=True, exist_ok=True)

# Database
DATABASE_PATH = DATA_DIR / "tasks.db"

# Display settings
DISPLAY_WIDTH = 160
DISPLAY_HEIGHT = 128

# GPIO pins (BCM numbering)
PIN_CLK = 17        # Encoder CLK (A)
PIN_DT = 27         # Encoder DT (B)
PIN_SW = 22         # Encoder switch
PIN_BACKLIGHT = 18  # Display backlight

# Timing
IDLE_TIMEOUT = 300          # 5 minutes before screen off
POLL_INTERVAL = 0.001       # 1ms polling for encoder
COMPLETING_DURATION = 0.5   # Animation duration in seconds

# HTTP API (for remote task management)
API_HOST = "0.0.0.0"
API_PORT = 8080

# Web URL for QR code (defaults to hostname:port)
def get_web_url():
    """Get the web URL for QR code scanning"""
    import socket
    hostname = os.environ.get("KIOSK_HOSTNAME", socket.gethostname())
    return os.environ.get("KIOSK_WEB_URL", f"http://{hostname}:{API_PORT}")

WEB_URL = get_web_url()

