#!/bin/bash
# Raspberry Pi Kiosk Installation Script
# Run this script on the Raspberry Pi to set up the kiosk

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USER_HOME="/home/shreyash"
KIOSK_URL="https://days-tracker-server-deployment.reverse-python.ts.net/"

# Detect Chromium package name based on OS
# - Debian 12+ (bookworm, trixie): chromium
# - Older Debian/Raspbian/Ubuntu: chromium-browser
detect_chromium_package() {
    if apt-cache show chromium &>/dev/null; then
        echo "chromium"
    elif apt-cache show chromium-browser &>/dev/null; then
        echo "chromium-browser"
    else
        echo ""
    fi
}

CHROMIUM_PACKAGE=$(detect_chromium_package)

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Raspberry Pi Kiosk Installer${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# Check if running as root for apt commands
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${YELLOW}Note: Some commands require sudo. You may be prompted for your password.${NC}"
    fi
}

# Step 1: Update system and install dependencies
install_dependencies() {
    echo -e "${GREEN}[1/6] Installing dependencies...${NC}"

    sudo apt update

    # Detect chromium package if not already detected
    if [[ -z "$CHROMIUM_PACKAGE" ]]; then
        CHROMIUM_PACKAGE=$(detect_chromium_package)
    fi

    if [[ -z "$CHROMIUM_PACKAGE" ]]; then
        echo -e "${RED}Error: Could not find chromium or chromium-browser package${NC}"
        exit 1
    fi

    echo "Detected Chromium package: $CHROMIUM_PACKAGE"

    # Build tools for fbcp
    sudo apt install -y cmake git build-essential

    # X server and browser
    sudo apt install -y \
        xserver-xorg \
        xinit \
        x11-xserver-utils \
        "$CHROMIUM_PACKAGE" \
        unclutter \
        xdotool

    # Python dependencies for encoder
    sudo apt install -y \
        python3-pip \
        python3-gpiozero \
        python3-lgpio

    echo -e "${GREEN}Dependencies installed!${NC}"
}

# Step 2: Check if fbcp is needed
install_fbcp() {
    echo -e "${GREEN}[2/6] Checking framebuffer configuration...${NC}"

    # Check if TFT is already the primary framebuffer
    FB0_NAME=$(cat /sys/class/graphics/fb0/name 2>/dev/null || echo "unknown")

    if [[ "$FB0_NAME" == *"st7735"* ]] || [[ "$FB0_NAME" == *"fbtft"* ]]; then
        echo "TFT display is primary framebuffer (fb0 = $FB0_NAME)"
        echo "fbcp is NOT needed - X will render directly to TFT"
        echo -e "${GREEN}Framebuffer configured correctly!${NC}"
        return
    fi

    # If there's a secondary framebuffer, we might need fbcp
    if [[ -e /dev/fb1 ]]; then
        echo "Secondary framebuffer detected - fbcp may be needed"
        echo -e "${YELLOW}Note: fbcp requires libraspberrypi-dev which may not be available on Debian 13${NC}"
        echo "Skipping fbcp build - will try without it first"
    else
        echo "Single framebuffer setup detected"
    fi

    echo -e "${GREEN}Framebuffer check complete!${NC}"
}

# Step 3: Copy scripts to home directory
install_scripts() {
    echo -e "${GREEN}[3/6] Installing kiosk scripts...${NC}"

    # Copy kiosk script
    cp "${SCRIPT_DIR}/kiosk.sh" "${USER_HOME}/kiosk.sh"
    chmod +x "${USER_HOME}/kiosk.sh"

    # Copy encoder script
    cp "${SCRIPT_DIR}/encoder.py" "${USER_HOME}/encoder.py"
    chmod +x "${USER_HOME}/encoder.py"

    # Set ownership
    chown shreyash:shreyash "${USER_HOME}/kiosk.sh" "${USER_HOME}/encoder.py"

    echo -e "${GREEN}Scripts installed!${NC}"
}

# Step 4: Install systemd services
install_services() {
    echo -e "${GREEN}[4/6] Installing systemd services...${NC}"

    # Copy service files
    sudo cp "${SCRIPT_DIR}/kiosk.service" /etc/systemd/system/
    sudo cp "${SCRIPT_DIR}/encoder.service" /etc/systemd/system/

    # Reload systemd
    sudo systemctl daemon-reload

    # Enable services (but don't start yet)
    sudo systemctl enable kiosk.service
    sudo systemctl enable encoder.service

    echo -e "${GREEN}Services installed and enabled!${NC}"
}

# Step 5: Update config.txt
update_config() {
    echo -e "${GREEN}[5/6] Updating boot configuration...${NC}"

    CONFIG_FILE="/boot/firmware/config.txt"
    BACKUP_FILE="/boot/firmware/config.txt.backup.$(date +%Y%m%d%H%M%S)"

    # Backup existing config
    sudo cp "$CONFIG_FILE" "$BACKUP_FILE"
    echo "Backed up config.txt to $BACKUP_FILE"

    # Remove old st7735r overlay if present
    sudo sed -i '/dtoverlay=st7735r/d' "$CONFIG_FILE"
    sudo sed -i '/gpio=18=op,dh/d' "$CONFIG_FILE"

    # Check if our settings already exist
    if grep -q "adafruit18" "$CONFIG_FILE"; then
        echo "Display overlay already configured, skipping..."
    else
        # Add display configuration
        echo "" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "# ST7735 TFT Display (added by kiosk installer)" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18" | sudo tee -a "$CONFIG_FILE" > /dev/null
    fi

    # Add framebuffer settings if not present
    if ! grep -q "hdmi_cvt=160 128" "$CONFIG_FILE"; then
        echo "" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "# Framebuffer settings for kiosk (added by installer)" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "hdmi_force_hotplug=1" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "hdmi_cvt=160 128 60 1 0 0 0" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "hdmi_group=2" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "hdmi_mode=87" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "framebuffer_width=160" | sudo tee -a "$CONFIG_FILE" > /dev/null
        echo "framebuffer_height=128" | sudo tee -a "$CONFIG_FILE" > /dev/null
    fi

    echo -e "${GREEN}Boot configuration updated!${NC}"
}

# Step 6: Test connectivity
test_connectivity() {
    echo -e "${GREEN}[6/6] Testing Tailscale connectivity...${NC}"

    # Check Tailscale status
    if command -v tailscale &> /dev/null; then
        echo "Tailscale status:"
        tailscale status | head -5
        echo ""
    else
        echo -e "${YELLOW}Warning: Tailscale not found${NC}"
    fi

    # Test URL
    echo "Testing connection to kiosk URL..."
    if curl -s --max-time 10 -I "$KIOSK_URL" | head -1; then
        echo -e "${GREEN}Connection successful!${NC}"
    else
        echo -e "${YELLOW}Warning: Could not reach $KIOSK_URL${NC}"
        echo "Make sure Tailscale is connected and the server is running."
    fi
}

# Main installation
main() {
    check_root

    echo "This script will:"
    echo "  1. Install required packages (X server, Chromium, etc.)"
    echo "  2. Build and install fbcp"
    echo "  3. Copy kiosk and encoder scripts"
    echo "  4. Install systemd services"
    echo "  5. Update /boot/firmware/config.txt"
    echo "  6. Test Tailscale connectivity"
    echo ""
    echo -e "${YELLOW}A reboot will be required after installation.${NC}"
    echo ""
    read -p "Continue? [y/N] " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi

    install_dependencies
    install_fbcp
    install_scripts
    install_services
    update_config
    test_connectivity

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Installation Complete!${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Reboot the Pi: sudo reboot"
    echo "  2. After reboot, test the display:"
    echo "     ls -la /dev/fb*  (should show fb0 and fb1)"
    echo "     cat /dev/urandom > /dev/fb1  (should show static)"
    echo "  3. The kiosk should start automatically"
    echo ""
    echo "To check service status:"
    echo "  sudo systemctl status kiosk"
    echo "  sudo systemctl status encoder"
    echo ""
    echo "To view logs:"
    echo "  journalctl -u kiosk -f"
    echo "  journalctl -u encoder -f"
    echo ""

    read -p "Reboot now? [y/N] " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo reboot
    fi
}

main "$@"
