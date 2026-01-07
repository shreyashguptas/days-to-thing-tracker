#!/bin/bash
# Raspberry Pi Kiosk Startup Script
# Starts fbcp and launches Chromium in kiosk mode

set -e

# Configuration
KIOSK_URL="${KIOSK_URL:-https://days-tracker-server-deployment.reverse-python.ts.net/}"
DISPLAY_WIDTH=160
DISPLAY_HEIGHT=128

# Detect Chromium binary (chromium on Debian 12+, chromium-browser on older)
detect_chromium_binary() {
    if command -v chromium &> /dev/null; then
        echo "chromium"
    elif command -v chromium-browser &> /dev/null; then
        echo "chromium-browser"
    else
        echo ""
    fi
}

CHROMIUM_BIN=$(detect_chromium_binary)
if [[ -z "$CHROMIUM_BIN" ]]; then
    echo "ERROR: Chromium not found. Please install chromium or chromium-browser."
    exit 1
fi

echo "Starting Raspberry Pi Kiosk..."
echo "Target URL: $KIOSK_URL"

# Check if fbcp is needed (only if TFT is secondary framebuffer)
# On Debian 13 with fbtft, TFT is usually fb0 so fbcp isn't needed
if [[ -e /dev/fb1 ]] && command -v fbcp &> /dev/null; then
    echo "Starting fbcp (mirroring fb0 to fb1)..."
    /usr/local/bin/fbcp &
    sleep 1
else
    echo "TFT is primary framebuffer - fbcp not needed"
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
echo "Launching Chromium ($CHROMIUM_BIN)..."
exec "$CHROMIUM_BIN" \
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
