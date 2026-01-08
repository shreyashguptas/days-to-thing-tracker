#!/bin/bash
#
# Days Tracker Kiosk - Deployment Script
#
# Run this script on the Raspberry Pi to manage the kiosk installation.
#
# Usage:
#   ./deploy.sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
RUST_DIR="$SCRIPT_DIR/rust/kiosk-core"
DATA_DIR="$SCRIPT_DIR/data"

echo -e "${BLUE}"
echo "╔════════════════════════════════════════════╗"
echo "║       Days Tracker Kiosk Deployment        ║"
echo "╚════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if running on Pi
check_pi() {
    if [ ! -f /proc/device-tree/model ]; then
        echo -e "${YELLOW}Warning: Not running on a Raspberry Pi${NC}"
        echo "Some features may not work correctly."
        echo ""
    fi
}

# Show menu
show_menu() {
    echo -e "${GREEN}What would you like to do?${NC}"
    echo ""
    echo "  1) Quick Update     - Pull changes & restart services (Python only)"
    echo "  2) Full Update      - Pull changes, rebuild Rust, restart services"
    echo "  3) Fresh Install    - First-time setup (installs everything)"
    echo "  4) Restart Services - Just restart kiosk and API services"
    echo "  5) View Logs        - Show recent kiosk logs"
    echo "  6) Check Status     - Show service status and system info"
    echo "  7) Boot Config      - Show instructions for boot configuration"
    echo "  8) Exit"
    echo ""
    read -p "Enter choice [1-8]: " choice
}

# Pull latest changes
pull_changes() {
    echo -e "${BLUE}Pulling latest changes from GitHub...${NC}"
    cd "$SCRIPT_DIR"
    git pull
    echo -e "${GREEN}✓ Changes pulled${NC}"
}

# Activate virtual environment
activate_venv() {
    if [ -d "$VENV_DIR" ]; then
        source "$VENV_DIR/bin/activate"
    else
        echo -e "${RED}Virtual environment not found. Run Fresh Install first.${NC}"
        exit 1
    fi
}

# Build Rust library
build_rust() {
    echo -e "${BLUE}Building Rust library (this may take 5-10 minutes)...${NC}"
    cd "$RUST_DIR"
    maturin build --release
    # Wheel is built to workspace root target dir, not kiosk-core target dir
    pip install "$SCRIPT_DIR"/target/wheels/kiosk_core-*.whl --force-reinstall
    cd "$SCRIPT_DIR"
    echo -e "${GREEN}✓ Rust library built and installed${NC}"
}

# Update systemd services
update_services() {
    echo -e "${BLUE}Updating systemd services...${NC}"
    sudo cp "$SCRIPT_DIR/systemd/kiosk-tracker.service" /etc/systemd/system/
    sudo cp "$SCRIPT_DIR/systemd/kiosk-api.service" /etc/systemd/system/
    sudo systemctl daemon-reload
    echo -e "${GREEN}✓ Service files updated${NC}"
}

# Restart services
restart_services() {
    echo -e "${BLUE}Restarting services...${NC}"
    sudo systemctl restart kiosk-tracker
    sudo systemctl restart kiosk-api
    echo -e "${GREEN}✓ Services restarted${NC}"
}

# Enable services
enable_services() {
    echo -e "${BLUE}Enabling services to start at boot...${NC}"
    sudo systemctl enable kiosk-tracker
    sudo systemctl enable kiosk-api
    echo -e "${GREEN}✓ Services enabled${NC}"
}

# Quick update (Python only)
quick_update() {
    echo -e "${YELLOW}=== Quick Update ===${NC}"
    echo ""
    pull_changes
    update_services
    restart_services
    echo ""
    echo -e "${GREEN}✓ Quick update complete!${NC}"
}

# Full update (with Rust rebuild)
full_update() {
    echo -e "${YELLOW}=== Full Update ===${NC}"
    echo ""
    pull_changes
    activate_venv
    build_rust
    update_services
    restart_services
    echo ""
    echo -e "${GREEN}✓ Full update complete!${NC}"
}

# Fresh install
fresh_install() {
    echo -e "${YELLOW}=== Fresh Install ===${NC}"
    echo ""
    echo -e "${RED}This will set up everything from scratch.${NC}"
    read -p "Continue? [y/N]: " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        return
    fi

    echo ""
    echo -e "${BLUE}[1/7] Updating system packages...${NC}"
    sudo apt-get update
    sudo apt-get install -y \
        python3-pip \
        python3-venv \
        python3-dev \
        build-essential \
        pkg-config \
        libffi-dev \
        git \
        curl

    echo ""
    echo -e "${BLUE}[2/7] Checking Rust installation...${NC}"
    if ! command -v cargo &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo "Rust already installed: $(cargo --version)"
    fi
    # Make sure cargo is in path
    source "$HOME/.cargo/env" 2>/dev/null || true

    echo ""
    echo -e "${BLUE}[3/7] Setting up Python virtual environment...${NC}"
    if [ ! -d "$VENV_DIR" ]; then
        python3 -m venv "$VENV_DIR"
    fi
    source "$VENV_DIR/bin/activate"
    pip install --upgrade pip
    pip install -r "$SCRIPT_DIR/python/requirements.txt"
    pip install maturin

    echo ""
    echo -e "${BLUE}[4/7] Building Rust library...${NC}"
    build_rust

    echo ""
    echo -e "${BLUE}[5/7] Creating data directory...${NC}"
    mkdir -p "$DATA_DIR"

    echo ""
    echo -e "${BLUE}[6/7] Installing systemd services...${NC}"
    update_services
    enable_services

    echo ""
    echo -e "${BLUE}[7/7] Starting services...${NC}"
    restart_services

    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║         Fresh Install Complete!            ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}IMPORTANT: You still need to configure boot settings!${NC}"
    echo "Run this script again and select option 7 for instructions."
    echo ""
}

# View logs
view_logs() {
    echo -e "${YELLOW}=== Recent Kiosk Logs ===${NC}"
    echo "(Press Ctrl+C to exit)"
    echo ""
    sudo journalctl -u kiosk-tracker -f
}

# Check status
check_status() {
    echo -e "${YELLOW}=== System Status ===${NC}"
    echo ""

    echo -e "${BLUE}Kiosk Service:${NC}"
    sudo systemctl status kiosk-tracker --no-pager -l 2>/dev/null || echo "Not running"
    echo ""

    echo -e "${BLUE}API Service:${NC}"
    sudo systemctl status kiosk-api --no-pager -l 2>/dev/null || echo "Not running"
    echo ""

    echo -e "${BLUE}Framebuffer:${NC}"
    if [ -e /dev/fb0 ]; then
        echo "  /dev/fb0 exists"
        echo "  Driver: $(cat /sys/class/graphics/fb0/name 2>/dev/null || echo 'unknown')"
    else
        echo "  /dev/fb0 NOT FOUND - display driver not configured!"
    fi
    echo ""

    echo -e "${BLUE}Network:${NC}"
    echo "  IP: $(hostname -I | awk '{print $1}')"
    echo "  Hostname: $(hostname)"
    echo "  API URL: http://$(hostname -I | awk '{print $1}'):8080"
    echo ""

    echo -e "${BLUE}Git Status:${NC}"
    cd "$SCRIPT_DIR"
    git log -1 --format="  Last commit: %h - %s (%cr)"
    echo ""
}

# Show boot config instructions
show_boot_config() {
    echo -e "${YELLOW}=== Boot Configuration Instructions ===${NC}"
    echo ""
    echo "These changes need to be made manually (one-time setup):"
    echo ""
    echo -e "${BLUE}1. Display Driver (config.txt)${NC}"
    echo "   Edit: sudo nano /boot/firmware/config.txt"
    echo "   Add these lines at the end:"
    echo ""
    echo -e "${GREEN}   # Enable SPI"
    echo "   dtparam=spi=on"
    echo ""
    echo "   # ST7735 160x128 TFT Display"
    echo "   dtoverlay=adafruit18,dc_pin=25,reset_pin=24,speed=32000000,rotate=90${NC}"
    echo ""
    echo -e "${BLUE}2. Disable Console on TFT (cmdline.txt)${NC}"
    echo "   Edit: sudo nano /boot/firmware/cmdline.txt"
    echo "   Add to the END of the single line (don't create new line):"
    echo ""
    echo -e "${GREEN}   fbcon=map:10${NC}"
    echo ""
    echo "   This prevents the login screen from appearing on the TFT."
    echo ""
    echo -e "${BLUE}3. Reboot to apply changes${NC}"
    echo "   Run: sudo reboot"
    echo ""
    echo -e "${YELLOW}Current config.txt display settings:${NC}"
    grep -E "(spi|dtoverlay.*adafruit|dtoverlay.*st7735)" /boot/firmware/config.txt 2>/dev/null || echo "   (no display overlay found)"
    echo ""
    echo -e "${YELLOW}Current cmdline.txt:${NC}"
    if grep -q "fbcon=map" /boot/firmware/cmdline.txt 2>/dev/null; then
        echo -e "   ${GREEN}fbcon setting found ✓${NC}"
    else
        echo -e "   ${RED}fbcon=map:10 NOT found - login screen will appear on TFT${NC}"
    fi
    echo ""
}

# Main
check_pi
show_menu

case $choice in
    1)
        quick_update
        ;;
    2)
        full_update
        ;;
    3)
        fresh_install
        ;;
    4)
        restart_services
        ;;
    5)
        view_logs
        ;;
    6)
        check_status
        ;;
    7)
        show_boot_config
        ;;
    8)
        echo "Goodbye!"
        ;;
    *)
        echo -e "${RED}Invalid choice.${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Done!${NC}"
