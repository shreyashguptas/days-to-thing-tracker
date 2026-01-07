#!/usr/bin/env python3
"""
setup.py - Setup and deploy Days to Thing Tracker

Usage:
    python setup.py              # Show deployment menu
    python setup.py --init       # First-time setup
    python setup.py --kiosk      # Kiosk setup (Raspberry Pi display)
"""

import os
import sys
import subprocess
import shutil
import getpass
import time
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.absolute()
ENV_FILE = PROJECT_ROOT / ".env"
DOCKER_STATE_DIR = PROJECT_ROOT / ".docker" / "tailscale" / "state"
KIOSK_DEPLOYMENT_DIR = PROJECT_ROOT / "raspberry-pi-deployment"

# Kiosk configuration
KIOSK_URL = "https://days-tracker-server-deployment.reverse-python.ts.net/?kiosk=true"
PI_USER_HOME = Path("/home/shreyash")


def has_npm() -> bool:
    """Check if npm is available."""
    return shutil.which("npm") is not None


def run(cmd: list, show_output: bool = True) -> bool:
    """Run a command and return success status."""
    if show_output:
        result = subprocess.run(cmd, cwd=PROJECT_ROOT)
    else:
        result = subprocess.run(cmd, cwd=PROJECT_ROOT, capture_output=True)
    return result.returncode == 0


def print_header(text: str):
    """Print a section header."""
    print(f"\n{'='*50}")
    print(f"  {text}")
    print(f"{'='*50}\n")


def read_env_file() -> dict:
    """Read existing .env file."""
    config = {}
    if ENV_FILE.exists():
        with open(ENV_FILE, "r") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, value = line.split("=", 1)
                    config[key.strip()] = value.strip()
    return config


def save_env_file(config: dict):
    """Save .env file."""
    lines = [
        "# Days to Thing Tracker Configuration",
        "",
        f"TS_AUTHKEY={config.get('TS_AUTHKEY', '')}",
        f"TS_HOSTNAME={config.get('TS_HOSTNAME', 'days-tracker')}",
        f"DATABASE_URL={config.get('DATABASE_URL', 'file:./data/tasks.db')}",
    ]
    with open(ENV_FILE, "w") as f:
        f.write("\n".join(lines) + "\n")
    os.chmod(ENV_FILE, 0o600)


def configure_tailscale_serve():
    """Configure Tailscale Serve to proxy HTTPS to the app."""
    print("\nConfiguring Tailscale Serve...")

    # Wait for Tailscale to connect (up to 30 seconds)
    for i in range(30):
        result = subprocess.run(
            ["docker", "exec", "days-tracker-tailscale", "tailscale", "status"],
            capture_output=True, text=True
        )
        if result.returncode == 0 and "100." in result.stdout:
            break
        # Check if needs approval
        if "not yet approved" in result.stderr.lower():
            print("\nMachine needs approval in Tailscale admin console.")
            print("Go to: https://login.tailscale.com/admin/machines")
            print("Find the new device and approve it.")
            input("\nPress Enter after approving...")
            continue
        time.sleep(1)
    else:
        print("Warning: Tailscale may not be connected yet")

    # Configure serve
    result = subprocess.run(
        ["docker", "exec", "days-tracker-tailscale",
         "tailscale", "serve", "--bg", "--https=443", "http://127.0.0.1:3000"],
        capture_output=True, text=True
    )

    # Check if needs approval (serve command can also fail for this)
    if "not yet approved" in result.stderr.lower():
        print("\nMachine needs approval in Tailscale admin console.")
        print("Go to: https://login.tailscale.com/admin/machines")
        print("Find the new device and approve it.")
        input("\nPress Enter after approving...")
        # Retry
        result = subprocess.run(
            ["docker", "exec", "days-tracker-tailscale",
             "tailscale", "serve", "--bg", "--https=443", "http://127.0.0.1:3000"],
            capture_output=True, text=True
        )

    if result.returncode == 0 or "already" in result.stderr.lower():
        # Get the serve status to show the URL
        status = subprocess.run(
            ["docker", "exec", "days-tracker-tailscale", "tailscale", "serve", "status"],
            capture_output=True, text=True
        )
        print("Tailscale Serve configured!")
        if status.stdout:
            print(status.stdout)
    else:
        print(f"Warning: Could not configure Tailscale Serve: {result.stderr}")


def configure_tailscale():
    """Configure Tailscale auth key and hostname."""
    print_header("Tailscale Configuration")

    existing = read_env_file()
    existing_key = existing.get("TS_AUTHKEY", "")
    existing_hostname = existing.get("TS_HOSTNAME", "days-tracker")

    # Configure hostname
    print(f"Current hostname: {existing_hostname}")
    print("(Use different names for different machines, e.g., days-tracker-mac, days-tracker-server)")
    choice = input("[K]eep current / [C]hange hostname? [K]: ").strip().lower()
    if choice == "c":
        new_hostname = input("Enter new hostname: ").strip()
        if new_hostname:
            existing["TS_HOSTNAME"] = new_hostname
            print(f"Hostname changed to: {new_hostname}")
        else:
            print("No hostname entered, keeping current.")
            existing["TS_HOSTNAME"] = existing_hostname
    else:
        print(f"Keeping hostname: {existing_hostname}")
        existing["TS_HOSTNAME"] = existing_hostname

    # Configure auth key
    if existing_key:
        masked = existing_key[:12] + "..." if len(existing_key) > 15 else "***"
        print(f"\nCurrent key: {masked}")
        choice = input("[K]eep current / [N]ew key / [S]kip? [K]: ").strip().lower()
        if choice == "n":
            print("\nGet a key from: https://login.tailscale.com/admin/settings/keys")
            new_key = getpass.getpass("Enter new Tailscale Auth Key: ")
            existing["TS_AUTHKEY"] = new_key
        elif choice != "s":
            print("Keeping existing key.")
    else:
        print("\nNo Tailscale key configured.")
        print("Get a key from: https://login.tailscale.com/admin/settings/keys")
        print("  - Create a reusable auth key")
        print("  - Suggested tag: tag:server")
        new_key = getpass.getpass("\nEnter Tailscale Auth Key (or Enter to skip): ")
        existing["TS_AUTHKEY"] = new_key

    existing["DATABASE_URL"] = "file:./data/tasks.db"
    save_env_file(existing)
    print("\nConfiguration saved!")


def deploy_restart():
    """Quick restart - just restart containers (for code changes bundled in image)."""
    print_header("Quick Restart")
    print("Restarting containers...")
    run(["docker", "compose", "restart"])
    print("\nDone! App restarted.")


def deploy_refresh():
    """Refresh - bring down and up (clears container state)."""
    print_header("Refresh Containers")
    print("Bringing containers down...")
    run(["docker", "compose", "down"])
    print("\nBringing containers up...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()
    print("\nDone! Containers refreshed.")


def deploy_rebuild():
    """Rebuild - rebuild Docker image and restart."""
    print_header("Rebuild Container")
    print("Stopping containers...")
    run(["docker", "compose", "down"])
    print("\nRebuilding image...")
    if not run(["docker", "compose", "build"]):
        print("\nBuild failed!")
        return
    print("\nStarting containers...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()
    print("\nDone! Container rebuilt and running.")


def deploy_full():
    """Full rebuild - npm install, prisma, rebuild container."""
    print_header("Full Rebuild")

    print("Stopping containers...")
    run(["docker", "compose", "down"])

    if has_npm():
        print("\nInstalling npm dependencies...")
        if not run(["npm", "install"]):
            print("npm install failed!")
            return

        print("\nGenerating Prisma client...")
        if not run(["npx", "prisma", "generate"]):
            print("Prisma generate failed!")
            return

        print("\nRunning database migrations...")
        run(["npx", "prisma", "migrate", "deploy"])
    else:
        print("\nNote: npm not found - skipping local npm/prisma")

    print("\nRebuilding Docker image...")
    if not run(["docker", "compose", "build"]):
        print("Docker build failed!")
        return

    print("\nStarting containers...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()
    print("\nDone! Full rebuild complete.")


def deploy_clean():
    """Clean rebuild - remove node_modules, rebuild everything."""
    print_header("Clean Rebuild")

    confirm = input("This will rebuild with --no-cache. Continue? [y/N]: ")
    if confirm.lower() != "y":
        print("Cancelled.")
        return

    print("Stopping containers...")
    run(["docker", "compose", "down"])

    if has_npm():
        print("\nRemoving node_modules...")
        node_modules = PROJECT_ROOT / "node_modules"
        if node_modules.exists():
            shutil.rmtree(node_modules)

        print("\nInstalling npm dependencies...")
        if not run(["npm", "install"]):
            print("npm install failed!")
            return

        print("\nGenerating Prisma client...")
        if not run(["npx", "prisma", "generate"]):
            print("Prisma generate failed!")
            return

        print("\nRunning database migrations...")
        run(["npx", "prisma", "migrate", "deploy"])
    else:
        print("\nNote: npm not found - skipping local npm/prisma")

    print("\nRebuilding Docker image (no cache)...")
    if not run(["docker", "compose", "build", "--no-cache"]):
        print("Docker build failed!")
        return

    print("\nStarting containers...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()
    print("\nDone! Clean rebuild complete.")


def show_logs():
    """Show container logs."""
    print_header("Container Logs")
    run(["docker", "compose", "logs", "-f", "--tail=50"])


def show_status():
    """Show container status."""
    print_header("Container Status")
    run(["docker", "compose", "ps"])


def stop_containers():
    """Stop all containers."""
    print_header("Stop Containers")
    run(["docker", "compose", "down"])
    print("Containers stopped.")


def reset_tailscale():
    """Reset Tailscale - clear all state and reconfigure."""
    print_header("Reset Tailscale")

    print("This will:")
    print("  1. Stop containers")
    print("  2. Delete all Tailscale state")
    print("  3. Require new auth key")
    print("  4. Re-register with Tailscale")
    print()
    print("WARNING: You must also delete the OLD device from Tailscale admin console!")
    print("         https://login.tailscale.com/admin/machines")
    print()

    confirm = input("Continue? [y/N]: ").strip().lower()
    if confirm != "y":
        print("Cancelled.")
        return

    # Stop containers
    print("\nStopping containers...")
    run(["docker", "compose", "down"])

    # Clear Tailscale state
    print("\nClearing Tailscale state...")
    if DOCKER_STATE_DIR.exists():
        shutil.rmtree(DOCKER_STATE_DIR)
    DOCKER_STATE_DIR.mkdir(parents=True, exist_ok=True)

    # Force new auth key
    existing = read_env_file()
    existing["TS_AUTHKEY"] = ""
    save_env_file(existing)

    print("\n" + "=" * 50)
    print("IMPORTANT: Before continuing, delete the old device")
    print("from your Tailscale admin console!")
    print("https://login.tailscale.com/admin/machines")
    print("=" * 50)
    input("\nPress Enter when done...")

    # Reconfigure
    configure_tailscale()

    # Restart
    print("\nStarting containers...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()

    print("\nTailscale reset complete!")


# =============================================================================
# KIOSK SETUP FUNCTIONS (Raspberry Pi Display)
# =============================================================================

def is_raspberry_pi() -> bool:
    """Check if running on a Raspberry Pi."""
    try:
        with open("/proc/cpuinfo", "r") as f:
            return "Raspberry Pi" in f.read() or "BCM" in f.read()
    except:
        return False


def detect_chromium_package() -> str:
    """Detect the correct Chromium package name."""
    result = subprocess.run(
        ["apt-cache", "show", "chromium"],
        capture_output=True, text=True
    )
    if result.returncode == 0:
        return "chromium"

    result = subprocess.run(
        ["apt-cache", "show", "chromium-browser"],
        capture_output=True, text=True
    )
    if result.returncode == 0:
        return "chromium-browser"

    return ""


def kiosk_install_dependencies() -> bool:
    """Install kiosk dependencies (apt packages)."""
    print("\n[1/4] Installing dependencies...")

    chromium_pkg = detect_chromium_package()
    if not chromium_pkg:
        print("Error: Could not find chromium or chromium-browser package")
        return False

    print(f"Detected Chromium package: {chromium_pkg}")

    # Update apt
    if not run(["sudo", "apt", "update"]):
        print("apt update failed!")
        return False

    # Build tools
    print("\nInstalling build tools...")
    if not run(["sudo", "apt", "install", "-y", "cmake", "git", "build-essential"]):
        return False

    # X server and browser
    print("\nInstalling X server, browser, and utilities...")
    packages = [
        "xserver-xorg", "xinit", "x11-xserver-utils",
        chromium_pkg, "unclutter", "xdotool"
    ]
    if not run(["sudo", "apt", "install", "-y"] + packages):
        return False

    # Python dependencies for encoder
    print("\nInstalling Python dependencies...")
    if not run(["sudo", "apt", "install", "-y", "python3-pip", "python3-gpiozero", "python3-lgpio"]):
        return False

    print("\nDependencies installed!")
    return True


def kiosk_install_scripts() -> bool:
    """Copy kiosk scripts to Pi home directory."""
    print("\n[2/4] Installing kiosk scripts...")

    # Check if deployment files exist
    kiosk_sh = KIOSK_DEPLOYMENT_DIR / "kiosk.sh"
    encoder_py = KIOSK_DEPLOYMENT_DIR / "encoder.py"
    fbdev_conf = KIOSK_DEPLOYMENT_DIR / "99-fbdev.conf"

    if not kiosk_sh.exists():
        print(f"Error: {kiosk_sh} not found!")
        print("Make sure you're running this from the project directory.")
        return False

    # Copy kiosk script
    dest_kiosk = PI_USER_HOME / "kiosk.sh"
    shutil.copy(kiosk_sh, dest_kiosk)
    os.chmod(dest_kiosk, 0o755)
    print(f"  Copied kiosk.sh to {dest_kiosk}")

    # Copy encoder script
    if encoder_py.exists():
        dest_encoder = PI_USER_HOME / "encoder.py"
        shutil.copy(encoder_py, dest_encoder)
        os.chmod(dest_encoder, 0o755)
        print(f"  Copied encoder.py to {dest_encoder}")

    # Set ownership
    user = os.environ.get("USER", "shreyash")
    run(["sudo", "chown", f"{user}:{user}", str(dest_kiosk)], show_output=False)
    if (PI_USER_HOME / "encoder.py").exists():
        run(["sudo", "chown", f"{user}:{user}", str(PI_USER_HOME / "encoder.py")], show_output=False)

    # Configure Chromium for low-memory kiosk mode
    print("  Configuring Chromium for low-memory kiosk mode...")
    run(["sudo", "mkdir", "-p", "/etc/chromium.d"], show_output=False)
    chromium_conf = "# Skip low memory warning for kiosk mode\nexport SKIP_MEMCHECK=1\n"
    subprocess.run(
        ["sudo", "tee", "/etc/chromium.d/99-kiosk"],
        input=chromium_conf, text=True, capture_output=True
    )

    # Configure X11 for TFT framebuffer
    print("  Configuring X11 for TFT framebuffer...")
    run(["sudo", "mkdir", "-p", "/etc/X11/xorg.conf.d"], show_output=False)
    if fbdev_conf.exists():
        run(["sudo", "cp", str(fbdev_conf), "/etc/X11/xorg.conf.d/"], show_output=False)

    print("Scripts installed!")
    return True


def kiosk_install_services() -> bool:
    """Install and enable systemd services."""
    print("\n[3/4] Installing systemd services...")

    kiosk_service = KIOSK_DEPLOYMENT_DIR / "kiosk.service"
    encoder_service = KIOSK_DEPLOYMENT_DIR / "encoder.service"

    if not kiosk_service.exists():
        print(f"Error: {kiosk_service} not found!")
        return False

    # Copy service files
    run(["sudo", "cp", str(kiosk_service), "/etc/systemd/system/"], show_output=False)
    print("  Copied kiosk.service")

    if encoder_service.exists():
        run(["sudo", "cp", str(encoder_service), "/etc/systemd/system/"], show_output=False)
        print("  Copied encoder.service")

    # Reload systemd
    run(["sudo", "systemctl", "daemon-reload"], show_output=False)

    # Enable services
    run(["sudo", "systemctl", "enable", "kiosk.service"], show_output=False)
    print("  Enabled kiosk.service")

    if encoder_service.exists():
        run(["sudo", "systemctl", "enable", "encoder.service"], show_output=False)
        print("  Enabled encoder.service")

    print("Services installed and enabled!")
    return True


def kiosk_test_connectivity() -> bool:
    """Test Tailscale connectivity to server."""
    print("\n[4/4] Testing connectivity...")

    # Check Tailscale status
    if shutil.which("tailscale"):
        print("\nTailscale status:")
        run(["tailscale", "status"])
    else:
        print("Warning: Tailscale not found")

    # Test URL
    print(f"\nTesting connection to: {KIOSK_URL}")
    result = subprocess.run(
        ["curl", "-s", "--max-time", "10", "-I", KIOSK_URL],
        capture_output=True, text=True
    )
    if result.returncode == 0 and "200" in result.stdout:
        print("Connection successful!")
        return True
    else:
        print("Warning: Could not reach server")
        print("Make sure Tailscale is connected and the server is running.")
        return False


def kiosk_show_config_instructions():
    """Show manual config.txt instructions."""
    print_header("Manual Configuration Required")

    print("Before running the automated setup, you need to edit /boot/firmware/config.txt")
    print("to configure the TFT display driver.\n")

    print("1. Open config.txt:")
    print("   sudo nano /boot/firmware/config.txt\n")

    print("2. REMOVE these lines (if present):")
    print("   dtoverlay=st7735r,dc_pin=25,reset_pin=24,speed=32000000,width=128,height=160")
    print("   gpio=18=op,dh\n")

    print("3. ADD these lines at the end:\n")
    print("   # ST7735 TFT Display")
    print("   dtoverlay=adafruit18,rotate=270,speed=32000000,dc_pin=25,reset_pin=24,led_pin=18")
    print()
    print("   # Framebuffer settings for kiosk")
    print("   hdmi_force_hotplug=1")
    print("   hdmi_cvt=160 128 60 1 0 0 0")
    print("   hdmi_group=2")
    print("   hdmi_mode=87")
    print("   framebuffer_width=160")
    print("   framebuffer_height=128\n")

    print("4. Save the file (Ctrl+O, Enter, Ctrl+X)\n")

    print("5. Reboot the Pi:")
    print("   sudo reboot\n")

    print("-" * 50)
    print("After rebooting, verify the display driver loaded:")
    print("   ls -la /dev/fb*")
    print("   (Should show fb0)")
    print("-" * 50)


def kiosk_setup_phase1():
    """Kiosk setup Phase 1: Show config instructions."""
    print_header("Kiosk Setup - Phase 1: Display Configuration")

    kiosk_show_config_instructions()

    print("\nOnce you have:")
    print("  1. Edited /boot/firmware/config.txt with the lines above")
    print("  2. Rebooted the Pi")
    print("  3. Verified /dev/fb0 exists")
    print()
    print("Run this setup again and select 'Phase 2' to continue.\n")


def kiosk_setup_phase2():
    """Kiosk setup Phase 2: Install software."""
    print_header("Kiosk Setup - Phase 2: Software Installation")

    # Verify we're on a Pi (or at least Linux)
    if sys.platform != "linux":
        print("Error: This must be run on the Raspberry Pi, not your development machine.")
        print(f"Current platform: {sys.platform}")
        return

    # Check framebuffer exists
    if not Path("/dev/fb0").exists():
        print("Warning: /dev/fb0 not found!")
        print("Did you complete Phase 1 and reboot?")
        confirm = input("\nContinue anyway? [y/N]: ").strip().lower()
        if confirm != "y":
            print("Cancelled. Complete Phase 1 first.")
            return

    print("This will:")
    print("  1. Install packages (X server, Chromium, xdotool, etc.)")
    print("  2. Copy kiosk and encoder scripts")
    print("  3. Install and enable systemd services")
    print("  4. Test connectivity to server")
    print()

    confirm = input("Continue? [y/N]: ").strip().lower()
    if confirm != "y":
        print("Cancelled.")
        return

    # Run installation steps
    if not kiosk_install_dependencies():
        print("\nDependency installation failed!")
        return

    if not kiosk_install_scripts():
        print("\nScript installation failed!")
        return

    if not kiosk_install_services():
        print("\nService installation failed!")
        return

    kiosk_test_connectivity()

    # Done!
    print_header("Kiosk Setup Complete!")
    print("The kiosk will start automatically on next reboot.\n")
    print("To start now without rebooting:")
    print("  sudo systemctl start kiosk")
    print("  sudo systemctl start encoder\n")
    print("To check status:")
    print("  sudo systemctl status kiosk")
    print("  sudo systemctl status encoder\n")
    print("To view logs:")
    print("  journalctl -u kiosk -f")
    print("  journalctl -u encoder -f\n")

    reboot = input("Reboot now? [y/N]: ").strip().lower()
    if reboot == "y":
        run(["sudo", "reboot"])


def kiosk_menu():
    """Show kiosk setup menu."""
    print_header("Kiosk Setup (Raspberry Pi Display)")

    print("Setup your Raspberry Pi as a kiosk display.\n")
    print("The setup has two phases:\n")
    print("  Phase 1: Configure the display driver (manual config.txt edit + reboot)")
    print("  Phase 2: Install software (automated)\n")
    print("-" * 50)
    print()
    print("  1. Phase 1 - Show config.txt instructions")
    print("  2. Phase 2 - Install kiosk software (run after Phase 1 + reboot)")
    print()
    print("  3. Test connectivity only")
    print("  4. Start/restart kiosk services")
    print("  5. Stop kiosk services")
    print("  6. View kiosk logs")
    print()
    print("  0. Back to main menu")
    print()

    choice = input("Choose [1-6, 0]: ").strip()

    if choice == "1":
        kiosk_setup_phase1()
    elif choice == "2":
        kiosk_setup_phase2()
    elif choice == "3":
        kiosk_test_connectivity()
    elif choice == "4":
        print("\nStarting kiosk services...")
        run(["sudo", "systemctl", "restart", "kiosk"])
        run(["sudo", "systemctl", "restart", "encoder"])
        print("Services started.")
    elif choice == "5":
        print("\nStopping kiosk services...")
        run(["sudo", "systemctl", "stop", "kiosk"])
        run(["sudo", "systemctl", "stop", "encoder"])
        print("Services stopped.")
    elif choice == "6":
        print("\nShowing kiosk logs (Ctrl+C to exit)...")
        run(["journalctl", "-u", "kiosk", "-u", "encoder", "-f", "--lines=50"])
    elif choice == "0":
        return
    else:
        print("Invalid choice.")


# =============================================================================
# SERVER SETUP FUNCTIONS
# =============================================================================

def first_time_setup():
    """First-time setup wizard."""
    print_header("First-Time Setup (Server)")

    # Create Tailscale state directory (data is handled by Docker volume)
    DOCKER_STATE_DIR.mkdir(parents=True, exist_ok=True)
    print("Created Tailscale state directory.")

    # Configure Tailscale
    configure_tailscale()

    # Check if npm is available (local dev) or Docker-only (server)
    if has_npm():
        print("\nInstalling npm dependencies...")
        if not run(["npm", "install"]):
            print("npm install failed!")
            return

        print("\nSetting up database...")
        run(["npx", "prisma", "generate"])
        run(["npx", "prisma", "migrate", "deploy"])
    else:
        print("\nNote: npm not found - using Docker-only deployment")
        print("(npm/prisma will run inside Docker container)")

    # Build and start
    print("\nBuilding Docker image...")
    if not run(["docker", "compose", "build"]):
        print("Docker build failed!")
        return

    print("\nStarting containers...")
    run(["docker", "compose", "up", "-d"])
    configure_tailscale_serve()

    print_header("Setup Complete!")
    print("Access the app via your Tailscale HTTPS URL shown above.")
    print("\nRun 'python setup.py' again to see deployment options.")


def server_menu():
    """Show server deployment menu."""
    print_header("Server Deployment")

    print("Deployment options:\n")
    print("  1. Restart      - Quick restart                    [DATA SAFE]")
    print("  2. Refresh      - Down + Up (reset container)      [DATA SAFE]")
    print("  3. Rebuild      - Rebuild Docker image + restart   [DATA SAFE]")
    print("  4. Full         - npm + prisma + rebuild image     [RUNS MIGRATIONS]")
    print("  5. Clean        - Delete node_modules + rebuild    [RUNS MIGRATIONS]")
    print()
    print("  6. Status       - Show container status")
    print("  7. Logs         - Show container logs")
    print("  8. Stop         - Stop containers")
    print("  9. Tailscale    - Configure Tailscale key/hostname")
    print("  r. Reset TS     - Clear Tailscale state & restart  [FIXES TLS ISSUES]")
    print()
    print("  0. Back to main menu")
    print()
    print("-" * 50)
    print("  [DATA SAFE]       = Your tasks are safe")
    print("  [RUNS MIGRATIONS] = Runs DB migrations (usually safe)")
    print()

    choice = input("Choose [1-9, r, 0]: ").strip().lower()

    actions = {
        "1": deploy_restart,
        "2": deploy_refresh,
        "3": deploy_rebuild,
        "4": deploy_full,
        "5": deploy_clean,
        "6": show_status,
        "7": show_logs,
        "8": stop_containers,
        "9": configure_tailscale,
        "r": reset_tailscale,
        "0": lambda: None,  # Return to main menu
    }

    action = actions.get(choice)
    if action:
        action()
    else:
        print("Invalid choice.")


def main_menu():
    """Show top-level menu: Server or Kiosk."""
    print_header("Days to Thing Tracker - Setup")

    print("What would you like to set up?\n")
    print("  1. Server  - Deploy/manage the Docker server")
    print("  2. Kiosk   - Set up Raspberry Pi display client")
    print()
    print("  0. Exit")
    print()

    choice = input("Choose [1-2, 0]: ").strip()

    if choice == "1":
        server_menu()
    elif choice == "2":
        kiosk_menu()
    elif choice == "0":
        pass
    else:
        print("Invalid choice.")


def main():
    if "--init" in sys.argv:
        first_time_setup()
        return

    if "--kiosk" in sys.argv:
        kiosk_menu()
        return

    # Check if first-time setup needed (only check .env, not node_modules)
    # Skip this check on Raspberry Pi (kiosk client doesn't need .env)
    if not ENV_FILE.exists() and not is_raspberry_pi():
        print("\nFirst-time server setup required.")
        confirm = input("Run setup now? [Y/n]: ").strip().lower()
        if confirm != "n":
            first_time_setup()
            return

    main_menu()


if __name__ == "__main__":
    main()
