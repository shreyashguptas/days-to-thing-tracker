#!/bin/bash
# Raspberry Pi Kiosk Startup Script
# Starts fbcp and launches Chromium in kiosk mode

set -e

# Configuration
KIOSK_URL="${KIOSK_URL:-https://days-tracker-server-deployment.reverse-python.ts.net/}"
DISPLAY_WIDTH=160
DISPLAY_HEIGHT=128

echo "Starting Raspberry Pi Kiosk..."
echo "Target URL: $KIOSK_URL"

# Start fbcp (framebuffer copy) in background
# This mirrors fb0 to fb1 (the TFT display)
if command -v fbcp &> /dev/null; then
    echo "Starting fbcp..."
    /usr/local/bin/fbcp &
    sleep 1
else
    echo "WARNING: fbcp not found. Display mirroring will not work."
fi

# Disable screen blanking and power management
echo "Disabling screen blanking..."
xset s off          # Disable screen saver
xset -dpms          # Disable Display Power Management Signaling
xset s noblank      # Don't blank the screen

# Hide the mouse cursor
echo "Hiding cursor..."
if command -v unclutter &> /dev/null; then
    unclutter -idle 0.1 -root &
fi

# Wait for network to be ready (important for Tailscale)
echo "Waiting for network..."
for i in {1..30}; do
    if curl -s --max-time 2 "$KIOSK_URL" > /dev/null 2>&1; then
        echo "Network ready!"
        break
    fi
    echo "Waiting for network... ($i/30)"
    sleep 2
done

# Launch Chromium in kiosk mode
echo "Launching Chromium..."
exec chromium-browser \
    --kiosk \
    --noerrdialogs \
    --disable-infobars \
    --disable-session-crashed-bubble \
    --disable-restore-session-state \
    --disable-translate \
    --no-first-run \
    --fast \
    --fast-start \
    --disable-features=TranslateUI \
    --disk-cache-dir=/dev/null \
    --window-size=${DISPLAY_WIDTH},${DISPLAY_HEIGHT} \
    --window-position=0,0 \
    --check-for-update-interval=31536000 \
    "$KIOSK_URL"
