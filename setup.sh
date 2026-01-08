#!/bin/bash
#
# Days Tracker Kiosk - Setup Script
#
# This script sets up everything needed to run the kiosk on a Raspberry Pi Zero 2 W.
# Run this after cloning the repository.
#
# Usage:
#   chmod +x setup.sh
#   ./setup.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "========================================"
echo "  Days Tracker Kiosk Setup"
echo "========================================"
echo ""

# Check if running on Raspberry Pi
if [ ! -f /proc/device-tree/model ]; then
    echo "Warning: Not running on a Raspberry Pi"
    echo "Some features may not work correctly."
    echo ""
fi

# 1. Update system packages
echo "[1/6] Updating system packages..."
sudo apt-get update
sudo apt-get install -y \
    python3-pip \
    python3-venv \
    build-essential \
    pkg-config \
    libffi-dev

# 2. Install Rust (if not already installed)
echo ""
echo "[2/6] Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust already installed: $(cargo --version)"
fi

# 3. Create Python virtual environment
echo ""
echo "[3/6] Setting up Python environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi
source venv/bin/activate
pip install --upgrade pip
pip install -r python/requirements.txt

# 4. Build Rust library
echo ""
echo "[4/6] Building Rust library..."
cd rust/kiosk-core
pip install maturin
maturin develop --release
cd "$SCRIPT_DIR"

# 5. Create data directory
echo ""
echo "[5/6] Creating data directory..."
mkdir -p data

# 6. Install systemd services
echo ""
echo "[6/6] Installing systemd services..."
sudo cp systemd/kiosk-tracker.service /etc/systemd/system/
sudo cp systemd/kiosk-api.service /etc/systemd/system/

# Update service files with correct paths
sudo sed -i "s|/home/pi/days-to-thing-tracker|$SCRIPT_DIR|g" /etc/systemd/system/kiosk-tracker.service
sudo sed -i "s|/home/pi/days-to-thing-tracker|$SCRIPT_DIR|g" /etc/systemd/system/kiosk-api.service

sudo systemctl daemon-reload

echo ""
echo "========================================"
echo "  Setup Complete!"
echo "========================================"
echo ""
echo "To start the kiosk:"
echo "  sudo systemctl start kiosk-tracker"
echo ""
echo "To start the API server (for remote task management):"
echo "  sudo systemctl start kiosk-api"
echo ""
echo "To enable at boot:"
echo "  sudo systemctl enable kiosk-tracker"
echo "  sudo systemctl enable kiosk-api"
echo ""
echo "To run manually (for testing):"
echo "  source venv/bin/activate"
echo "  python python/main.py"
echo ""
echo "API will be available at: http://$(hostname -I | awk '{print $1}'):8080"
echo ""
