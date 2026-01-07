#!/usr/bin/env python3
"""
setup.py - Setup and deploy Days to Thing Tracker

Usage:
    python setup.py              # Show deployment menu
    python setup.py --init       # First-time setup
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
        time.sleep(1)
    else:
        print("Warning: Tailscale may not be connected yet")

    # Configure serve
    result = subprocess.run(
        ["docker", "exec", "days-tracker-tailscale",
         "tailscale", "serve", "--bg", "--https=443", "http://127.0.0.1:3000"],
        capture_output=True, text=True
    )

    if result.returncode == 0:
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


def first_time_setup():
    """First-time setup wizard."""
    print_header("First-Time Setup")

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


def main_menu():
    """Show main deployment menu."""
    print_header("Days to Thing Tracker - Deploy")

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
    print("  0. Exit")
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
        "0": lambda: sys.exit(0),
    }

    action = actions.get(choice)
    if action:
        action()
    else:
        print("Invalid choice.")


def main():
    if "--init" in sys.argv:
        first_time_setup()
        return

    # Check if first-time setup needed (only check .env, not node_modules)
    if not ENV_FILE.exists():
        print("\nFirst-time setup required.")
        confirm = input("Run setup now? [Y/n]: ").strip().lower()
        if confirm != "n":
            first_time_setup()
            return

    main_menu()


if __name__ == "__main__":
    main()
