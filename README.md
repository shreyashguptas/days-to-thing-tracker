# Days to Thing Tracker

A simple, modern recurring task tracker that resets countdowns based on when tasks are actually completed, not on a fixed schedule.

## Features

- Track recurring tasks (water filter, maintenance, etc.)
- Smart countdown that resets based on actual completion date
- Tasks categorized by urgency: Overdue, Due Today, This Week, Coming Up
- Dark/Light theme toggle
- Mobile-friendly responsive design
- Docker deployment with Tailscale integration

## Quick Start

### Development

```bash
# Install dependencies
npm install

# Generate Prisma client
npx prisma generate

# Run migrations
npx prisma migrate deploy

# Start development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### Production (Docker with Tailscale)

```bash
# Run the setup wizard
python setup.py

# Or manually:
# 1. Copy .env.example to .env and add your Tailscale auth key
# 2. Build and start containers
docker compose up -d
```

Access via Tailscale Magic DNS: `http://days-tracker:3000`

## Setup Wizard

The `setup.py` script guides you through configuration:

```bash
python setup.py          # Interactive setup
python setup.py --check  # Verify configuration
python setup.py --reset  # Reset configuration
```

## Tech Stack

- **Frontend**: Next.js 16, React 19, TypeScript
- **Styling**: Tailwind CSS 4
- **Database**: SQLite with Prisma ORM
- **Container**: Docker with Tailscale sidecar

## Project Structure

```
├── src/
│   ├── app/              # Next.js app router pages & API
│   ├── components/       # React components
│   │   ├── ui/          # Reusable UI primitives
│   │   └── tasks/       # Task-specific components
│   ├── hooks/           # Custom React hooks
│   ├── lib/             # Utilities (prisma, date-utils)
│   └── types/           # TypeScript definitions
├── prisma/              # Database schema & migrations
├── docker/              # Dockerfile
├── docker-compose.yml   # Container orchestration
└── setup.py             # Setup wizard
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite database path | `file:./data/tasks.db` |
| `TS_AUTHKEY` | Tailscale auth key | - |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create task |
| GET | `/api/tasks/[id]` | Get task |
| PUT | `/api/tasks/[id]` | Update task |
| DELETE | `/api/tasks/[id]` | Archive task |
| POST | `/api/tasks/[id]/complete` | Mark complete |

## License

MIT
