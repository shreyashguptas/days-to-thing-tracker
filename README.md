# Days Tracker Kiosk

A standalone kiosk application for tracking recurring tasks on a Raspberry Pi Zero 2 W with a small TFT display and rotary encoder.

## Features

- **Standalone operation**: No server required, runs entirely on the Pi
- **Direct framebuffer rendering**: No browser overhead, instant response
- **Rotary encoder navigation**: Scroll through tasks, select actions
- **Screen timeout**: Automatic backlight control to save power
- **REST API**: Add/edit tasks from any device on your network
- **SQLite storage**: Local database with optional SMB backup

## Hardware Requirements

- Raspberry Pi Zero 2 W
- 160x128 TFT display (ST7735 or similar)
- KY-040 rotary encoder
- Wiring (see [docs/pinout.md](docs/pinout.md))

## Quick Start

1. **Flash Raspberry Pi OS Lite** to your SD card

2. **Clone this repository**:
   ```bash
   git clone https://github.com/shreyashguptas/days-to-thing-tracker.git
   cd days-to-thing-tracker
   ```

3. **Run setup**:
   ```bash
   chmod +x setup.sh
   ./setup.sh
   ```

4. **Start the kiosk**:
   ```bash
   sudo systemctl start kiosk-tracker
   ```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Python Application                    │
│  main.py - Event loop, business logic                   │
│  database.py - SQLite operations                        │
│  views.py - Navigation state machine                    │
│  api.py - REST API for remote management                │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 Rust Core (kiosk_core)                  │
│  encoder.rs - GPIO input handling (<10µs latency)       │
│  display.rs - Direct framebuffer rendering              │
│  renderer.rs - UI widgets and layout                    │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌────────────┴────────────┐
              │                         │
         GPIO 17,27,22             /dev/fb0
       (Rotary Encoder)          (TFT Display)
```

## Controls

| Action | Result |
|--------|--------|
| Rotate clockwise | Scroll down / Next item |
| Rotate counter-clockwise | Scroll up / Previous item |
| Short press | Select / Confirm |
| Long press (>0.5s) | Back / Settings |

## Project Structure

```
days-to-thing-tracker/
├── python/                 # Python application
│   ├── main.py            # Entry point
│   ├── database.py        # SQLite operations
│   ├── models.py          # Data models
│   ├── views.py           # Navigation state
│   ├── api.py             # REST API
│   └── config.py          # Configuration
├── rust/                   # Rust core library
│   └── kiosk-core/
│       └── src/
│           ├── lib.rs     # PyO3 module
│           ├── encoder.rs # GPIO handling
│           ├── display.rs # Framebuffer
│           ├── renderer.rs # UI rendering
│           └── theme.rs   # Colors
├── systemd/               # Service files
├── docs/                  # Documentation
├── setup.sh               # Installation script
└── Cargo.toml             # Rust workspace
```

## API Endpoints

The optional REST API runs on port 8080:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/:id` | Get task |
| PUT | `/api/tasks/:id` | Update task |
| DELETE | `/api/tasks/:id` | Delete task |
| POST | `/api/tasks/:id/complete` | Mark complete |
| GET | `/api/tasks/:id/history` | Get history |

### Create Task Example

```bash
curl -X POST http://pi.local:8080/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Change Air Filter",
    "recurrenceType": "daily",
    "recurrenceValue": 30,
    "nextDueDate": "2026-02-01"
  }'
```

## Configuration

Environment variables (set in systemd service or export):

| Variable | Default | Description |
|----------|---------|-------------|
| `KIOSK_DATA_DIR` | `./data` | Database location |
| `SMB_BACKUP_ENABLED` | `false` | Enable SMB backup |
| `SMB_SHARE_PATH` | - | SMB share path |

## Development

### Build Rust library locally

```bash
cd rust/kiosk-core
maturin develop
```

### Run without hardware (simulation mode)

```bash
python python/main.py
```

The application will run in simulation mode if GPIO/framebuffer aren't available.

## License

MIT
