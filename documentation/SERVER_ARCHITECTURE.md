# Days Tracker - Server Architecture Documentation

This document provides comprehensive technical documentation for the Days Tracker server deployment, including Docker containerization, Tailscale networking, database management, and the deployment orchestration system.

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Diagram](#architecture-diagram)
3. [Docker Container Setup](#docker-container-setup)
4. [Dockerfile Multi-Stage Build](#dockerfile-multi-stage-build)
5. [setup.py - Deployment Orchestration](#setuppy---deployment-orchestration)
6. [Tailscale Configuration](#tailscale-configuration)
7. [Database Setup - Prisma & SQLite](#database-setup---prisma--sqlite)
8. [API Routes & Endpoints](#api-routes--endpoints)
9. [Environment Variables](#environment-variables)
10. [Type Definitions](#type-definitions)
11. [Date Utilities & Calculations](#date-utilities--calculations)
12. [Deployment Workflows](#deployment-workflows)
13. [File Structure](#file-structure)
14. [Troubleshooting](#troubleshooting)

---

## Overview

Days Tracker is a recurring task tracking application built with:

- **Frontend**: Next.js 16 with React 19, Tailwind CSS 4
- **Backend**: Next.js API routes (App Router)
- **Database**: SQLite via Prisma ORM with LibSQL adapter
- **Deployment**: Docker containers with Tailscale networking
- **Orchestration**: Python setup.py script for simplified management

The application runs in Docker containers and is exposed via Tailscale Serve, providing secure HTTPS access from any device on your Tailscale network.

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         HOST MACHINE                                 │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    DOCKER ENVIRONMENT                          │ │
│  │                                                                 │ │
│  │  ┌─────────────────────────────────────────────────────────┐   │ │
│  │  │           TAILSCALE CONTAINER                           │   │ │
│  │  │           (days-tracker-tailscale)                      │   │ │
│  │  │                                                         │   │ │
│  │  │   ┌─────────────────────────────────────────────────┐   │   │ │
│  │  │   │  Tailscale Daemon                               │   │   │ │
│  │  │   │  - Connects to Tailscale network (100.x.x.x)    │   │   │ │
│  │  │   │  - Serves HTTPS on port 443                     │   │   │ │
│  │  │   │  - Auto TLS certificate management              │   │   │ │
│  │  │   └─────────────────────────────────────────────────┘   │   │ │
│  │  │                         │                               │   │ │
│  │  │                         │ Proxy to 127.0.0.1:3000       │   │ │
│  │  │                         ▼                               │   │ │
│  │  │   ┌─────────────────────────────────────────────────┐   │   │ │
│  │  │   │  APP CONTAINER (network: service:tailscale)     │   │   │ │
│  │  │   │  (days-tracker-app)                             │   │   │ │
│  │  │   │                                                 │   │   │ │
│  │  │   │   ┌─────────────────────────────────────────┐   │   │   │ │
│  │  │   │   │  Next.js Server (port 3000)             │   │   │   │ │
│  │  │   │   │  - Standalone production build          │   │   │   │ │
│  │  │   │   │  - API routes (/api/tasks/*)            │   │   │   │ │
│  │  │   │   │  - React SSR/CSR                        │   │   │   │ │
│  │  │   │   └─────────────────────────────────────────┘   │   │   │ │
│  │  │   │                         │                       │   │   │ │
│  │  │   │                         │ Prisma ORM            │   │   │ │
│  │  │   │                         ▼                       │   │   │ │
│  │  │   │   ┌─────────────────────────────────────────┐   │   │   │ │
│  │  │   │   │  SQLite Database                        │   │   │   │ │
│  │  │   │   │  /app/data/tasks.db                     │   │   │   │ │
│  │  │   │   │  (Docker volume: app-data)              │   │   │   │ │
│  │  │   │   └─────────────────────────────────────────┘   │   │   │ │
│  │  │   └─────────────────────────────────────────────────┘   │   │ │
│  │  └─────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  Persistent Storage:                                                 │
│  - .docker/tailscale/state/ (Tailscale credentials)                 │
│  - Docker volume: app-data (SQLite database)                        │
└─────────────────────────────────────────────────────────────────────┘

External Access:
┌─────────────────────────────────────────────────────────────────────┐
│  Any Tailscale Device                                               │
│  https://days-tracker-server-deployment.reverse-python.ts.net       │
│                              │                                       │
│                              ▼                                       │
│  Tailscale Network (encrypted WireGuard tunnel)                     │
│                              │                                       │
│                              ▼                                       │
│  Tailscale Serve (port 443) → Next.js App (port 3000)               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Docker Container Setup

### docker-compose.yml

**Location**: `docker-compose.yml` (project root)

```yaml
services:
  tailscale:
    image: tailscale/tailscale:latest
    container_name: days-tracker-tailscale
    hostname: ${TS_HOSTNAME:-days-tracker}
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    volumes:
      - ./.docker/tailscale/state:/var/lib/tailscale
      - /dev/net/tun:/dev/net/tun
    environment:
      - TS_AUTHKEY=${TS_AUTHKEY}
      - TS_STATE_DIR=/var/lib/tailscale
      - TS_USERSPACE=false
      - TS_ACCEPT_DNS=true
    restart: unless-stopped

  app:
    build:
      context: .
      dockerfile: docker/Dockerfile
    container_name: days-tracker-app
    network_mode: service:tailscale
    volumes:
      - app-data:/app/data
    depends_on:
      - tailscale
    restart: unless-stopped

volumes:
  app-data:
```

### Service Descriptions

| Service | Container Name | Purpose |
|---------|---------------|---------|
| `tailscale` | days-tracker-tailscale | Tailscale networking sidecar - handles VPN connection and HTTPS serving |
| `app` | days-tracker-app | Next.js application - serves the web app and API |

### Key Configuration Details

**Tailscale Container:**
- Uses kernel TUN device (`/dev/net/tun`) for native networking
- Requires `NET_ADMIN` and `SYS_MODULE` capabilities
- State persisted to `.docker/tailscale/state/` for credential retention
- Hostname configurable via `TS_HOSTNAME` environment variable

**App Container:**
- Uses `network_mode: service:tailscale` to share Tailscale's network namespace
- This allows the app to be accessible via Tailscale's 127.0.0.1 proxy
- Database persisted via named Docker volume `app-data`

---

## Dockerfile Multi-Stage Build

**Location**: `docker/Dockerfile`

The Dockerfile uses a multi-stage build for optimal image size and build caching.

### Stage 1: Dependencies

```dockerfile
FROM node:20-slim AS deps
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
```

- Installs OpenSSL (required by Prisma)
- Uses `npm ci` for reproducible dependency installation
- Caches node_modules for faster rebuilds

### Stage 2: Builder

```dockerfile
FROM node:20-slim AS builder
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npx prisma generate
ENV NEXT_TELEMETRY_DISABLED=1
ENV DATABASE_URL="file:./prisma/dev.db"
RUN npm run build
```

- Generates Prisma client from schema
- Builds Next.js in standalone mode
- Produces optimized production assets

### Stage 3: Runner (Production)

```dockerfile
FROM node:20-slim AS runner
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy built assets
COPY --from=builder /app/public ./public
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

# Copy Prisma files
COPY --from=builder /app/prisma ./prisma
COPY --from=builder /app/node_modules/.prisma ./node_modules/.prisma
COPY --from=builder /app/node_modules/@prisma ./node_modules/@prisma
COPY --from=builder /app/node_modules/prisma ./node_modules/prisma

# Create data directory
RUN mkdir -p /app/data && chown -R nextjs:nodejs /app/data

USER nextjs

EXPOSE 3000
ENV PORT=3000
ENV HOSTNAME="0.0.0.0"
ENV DATABASE_URL="file:/app/data/tasks.db"

# Run migrations, then start server
CMD ["sh", "-c", "node ./node_modules/prisma/build/index.js migrate deploy && node server.js"]
```

**Key Points:**
- Runs as non-root user `nextjs` for security
- Migrations run automatically on container startup
- Database stored at `/app/data/tasks.db` (persisted volume)

---

## setup.py - Deployment Orchestration

**Location**: `setup.py` (project root)

The setup.py script provides a comprehensive CLI for managing the deployment.

### Interactive Menu

```
╔══════════════════════════════════════════════╗
║     Days Tracker - Deployment Manager        ║
╠══════════════════════════════════════════════╣
║  1. Restart      - Quick restart [DATA SAFE] ║
║  2. Refresh      - Down + Up [DATA SAFE]     ║
║  3. Rebuild      - Rebuild Docker image      ║
║  4. Full         - npm + prisma + rebuild    ║
║  5. Clean        - Delete node_modules       ║
║  6. Status       - Show container status     ║
║  7. Logs         - Show container logs       ║
║  8. Stop         - Stop containers           ║
║  9. Tailscale    - Configure TS key/hostname ║
║  r. Reset TS     - Clear Tailscale state     ║
║  0. Exit                                     ║
╚══════════════════════════════════════════════╝
```

### Deployment Functions

| Function | Command | Data Safety | Use Case |
|----------|---------|-------------|----------|
| `deploy_restart()` | `docker compose restart` | SAFE | Quick restart after config changes |
| `deploy_refresh()` | `docker compose down && up -d` | SAFE | Clears container state, keeps volumes |
| `deploy_rebuild()` | `docker compose build` | SAFE | Rebuild image after code changes |
| `deploy_full()` | npm install + prisma + rebuild | MIGRATIONS | After adding dependencies or schema changes |
| `deploy_clean()` | Remove node_modules + rebuild --no-cache | MIGRATIONS | Clean slate rebuild |
| `reset_tailscale()` | Delete .docker/tailscale/state | - | Fix TLS/certificate issues |

### First-Time Setup Process

```python
def first_time_setup():
    # 1. Create Tailscale state directory
    os.makedirs(".docker/tailscale/state", exist_ok=True)

    # 2. Prompt for Tailscale auth key
    auth_key = input("Enter Tailscale auth key: ")

    # 3. Configure hostname
    hostname = input("Enter hostname [days-tracker]: ") or "days-tracker"

    # 4. Write .env file
    with open(".env", "w") as f:
        f.write(f"TS_AUTHKEY={auth_key}\n")
        f.write(f"TS_HOSTNAME={hostname}\n")

    # 5. Local npm setup (if available)
    if shutil.which("npm"):
        subprocess.run(["npm", "install"])
        subprocess.run(["npx", "prisma", "generate"])
        subprocess.run(["npx", "prisma", "migrate", "deploy"])

    # 6. Build and start containers
    subprocess.run(["docker", "compose", "build"])
    subprocess.run(["docker", "compose", "up", "-d"])

    # 7. Configure Tailscale Serve
    configure_tailscale_serve()
```

### Tailscale Serve Configuration

```python
def configure_tailscale_serve():
    # Wait for Tailscale to connect (up to 30 seconds)
    for _ in range(30):
        result = subprocess.run(
            ["docker", "exec", "days-tracker-tailscale", "tailscale", "status"],
            capture_output=True, text=True
        )
        if "100." in result.stdout:  # Tailscale IP assigned
            break
        time.sleep(1)

    # Check if device needs approval
    if "not yet approved" in result.stderr:
        print("Device needs approval in Tailscale admin console")
        print("Visit: https://login.tailscale.com/admin/machines")
        input("Press Enter after approving...")

    # Configure Serve to proxy HTTPS:443 -> HTTP:3000
    subprocess.run([
        "docker", "exec", "days-tracker-tailscale",
        "tailscale", "serve", "--bg", "--https=443", "http://127.0.0.1:3000"
    ])

    # Display access URL
    result = subprocess.run(
        ["docker", "exec", "days-tracker-tailscale", "tailscale", "status", "--json"],
        capture_output=True, text=True
    )
    dns_name = json.loads(result.stdout)["Self"]["DNSName"]
    print(f"Access URL: https://{dns_name}")
```

---

## Tailscale Configuration

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `TS_AUTHKEY` | Authentication key from Tailscale dashboard | `tskey-auth-kXYZ...` |
| `TS_HOSTNAME` | Device name in Tailscale network | `days-tracker-server` |
| `TS_STATE_DIR` | State persistence directory | `/var/lib/tailscale` |
| `TS_USERSPACE` | Use kernel networking (not userspace) | `false` |
| `TS_ACCEPT_DNS` | Accept Tailscale DNS settings | `true` |

### Auth Key Generation

1. Go to https://login.tailscale.com/admin/settings/keys
2. Generate a new auth key
3. Recommended settings:
   - **Reusable**: Yes (allows re-deployment)
   - **Tags**: `tag:server` (optional, for ACL policies)
   - **Expiration**: Set according to security requirements

### Tailscale Serve

Tailscale Serve provides HTTPS termination and proxying:

```bash
tailscale serve --bg --https=443 http://127.0.0.1:3000
```

- `--bg`: Run in background
- `--https=443`: Listen for HTTPS on port 443
- `http://127.0.0.1:3000`: Proxy to local Next.js app

**Benefits:**
- Automatic TLS certificate management
- No port forwarding or firewall configuration needed
- Accessible only from Tailscale network (secure by default)

### Device Approval

New devices require approval in the Tailscale admin console:

1. Device connects but shows "not yet approved"
2. Visit https://login.tailscale.com/admin/machines
3. Find the new device and click "Approve"
4. setup.py automatically retries connection

---

## Database Setup - Prisma & SQLite

### Prisma Configuration

**Location**: `prisma.config.ts`

```typescript
import { defineConfig } from 'prisma/config'

export default defineConfig({
  schema: "prisma/schema.prisma",
  migrations: "prisma/migrations",
  datasource: {
    url: process.env["DATABASE_URL"]
  }
})
```

### Schema Definition

**Location**: `prisma/schema.prisma`

```prisma
generator client {
  provider = "prisma-client-js"
}

datasource db {
  provider = "sqlite"
  url      = env("DATABASE_URL")
}

model Task {
  id              String           @id @default(cuid())
  name            String
  description     String?
  intervalValue   Int
  intervalUnit    String           // "days", "weeks", "months"
  lastCompletedAt DateTime?
  createdAt       DateTime         @default(now())
  updatedAt       DateTime         @updatedAt
  isArchived      Boolean          @default(false)
  completions     TaskCompletion[]

  @@index([isArchived])
  @@index([lastCompletedAt])
}

model TaskCompletion {
  id          String   @id @default(cuid())
  taskId      String
  completedAt DateTime @default(now())
  task        Task     @relation(fields: [taskId], references: [id], onDelete: Cascade)

  @@index([taskId])
  @@index([completedAt])
}

model Settings {
  id        String   @id @default("singleton")
  theme     String   @default("system")
  updatedAt DateTime @updatedAt
}
```

### Database Models

| Model | Purpose | Key Fields |
|-------|---------|------------|
| **Task** | Recurring task definition | name, intervalValue, intervalUnit, lastCompletedAt |
| **TaskCompletion** | Completion history | taskId, completedAt |
| **Settings** | App settings (singleton) | theme |

### Prisma Client Initialization

**Location**: `src/lib/prisma.ts`

```typescript
import { PrismaClient } from "@prisma/client";
import { PrismaLibSQL } from "@prisma/adapter-libsql";
import { createClient } from "@libsql/client";

const libsql = createClient({
  url: process.env.DATABASE_URL || "file:./prisma/dev.db",
});

const adapter = new PrismaLibSQL(libsql);

const prismaClientSingleton = () => {
  return new PrismaClient({ adapter });
};

// Prevent multiple instances in development
const globalForPrisma = globalThis as unknown as {
  prisma: ReturnType<typeof prismaClientSingleton> | undefined;
};

export const prisma = globalForPrisma.prisma ?? prismaClientSingleton();

if (process.env.NODE_ENV !== "production") {
  globalForPrisma.prisma = prisma;
}
```

### Migrations

| Migration | Purpose |
|-----------|---------|
| `20260106194646_init` | Create Task and Settings tables |
| `20260106204935_add_completion_history` | Add TaskCompletion table |

**Running Migrations:**
- Automatically on container startup: `prisma migrate deploy`
- Manually: `npx prisma migrate deploy`

---

## API Routes & Endpoints

### Base URL

Development: `http://localhost:3000/api/tasks`
Production: `https://{hostname}.{tailnet}.ts.net/api/tasks`

### Endpoint Reference

#### GET /api/tasks

List all active tasks grouped by urgency.

**Response:**
```typescript
{
  overdue: TaskWithDue[],   // daysUntilDue < 0
  today: TaskWithDue[],     // daysUntilDue === 0
  thisWeek: TaskWithDue[],  // 0 < daysUntilDue <= 7
  upcoming: TaskWithDue[]   // daysUntilDue > 7
}
```

#### POST /api/tasks

Create a new recurring task.

**Request Body:**
```typescript
{
  name: string,              // Required
  description?: string,
  intervalValue: number,     // Required, >= 1
  intervalUnit: "days" | "weeks" | "months"  // Required
}
```

**Response:** Created Task (201)

#### GET /api/tasks/[id]

Get a single task with due date information.

**Response:** TaskWithDue

#### PUT /api/tasks/[id]

Update task properties.

**Request Body:**
```typescript
{
  name?: string,
  description?: string,
  intervalValue?: number,
  intervalUnit?: "days" | "weeks" | "months"
}
```

**Response:** Updated Task

#### DELETE /api/tasks/[id]

Soft delete (archive) a task.

**Response:** `{ success: true }`

#### POST /api/tasks/[id]/complete

Mark a task as completed.

**Response:**
```typescript
{
  success: true,
  task: {
    ...Task,
    lastCompletedAt: string,
    nextDueDate: string,
    daysUntilDue: number
  }
}
```

#### GET /api/tasks/[id]/history

Get completion history for a task.

**Response:**
```typescript
{
  completions: [
    { id: string, completedAt: string }
  ]
}
```

---

## Environment Variables

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `TS_AUTHKEY` | Tailscale authentication key | `tskey-auth-xxx` |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TS_HOSTNAME` | `days-tracker` | Device name in Tailscale |
| `DATABASE_URL` | `file:/app/data/tasks.db` | SQLite database path |
| `NODE_ENV` | `production` (in Docker) | Runtime environment |

### .env File Template

```bash
# Tailscale Configuration
TS_AUTHKEY=tskey-auth-your-key-here
TS_HOSTNAME=days-tracker-server

# Database (optional, has sensible default)
DATABASE_URL=file:./data/tasks.db
```

---

## Type Definitions

**Location**: `src/types/index.ts`

```typescript
export type IntervalUnit = "days" | "weeks" | "months";
export type UrgencyLevel = "overdue" | "today" | "this-week" | "upcoming";

export interface Task {
  id: string;
  name: string;
  description: string | null;
  intervalValue: number;
  intervalUnit: IntervalUnit;
  lastCompletedAt: string | null;
  createdAt: string;
  updatedAt: string;
  isArchived: boolean;
}

export interface TaskWithDue extends Task {
  nextDueDate: string;      // ISO date string
  daysUntilDue: number;     // Negative = overdue
  urgency: UrgencyLevel;
}

export interface TasksResponse {
  overdue: TaskWithDue[];
  today: TaskWithDue[];
  thisWeek: TaskWithDue[];
  upcoming: TaskWithDue[];
}

export interface CreateTaskInput {
  name: string;
  description?: string;
  intervalValue: number;
  intervalUnit: IntervalUnit;
}

export interface UpdateTaskInput extends Partial<CreateTaskInput> {}
```

---

## Date Utilities & Calculations

**Location**: `src/lib/date-utils.ts`

### Core Functions

| Function | Purpose |
|----------|---------|
| `calculateNextDueDate()` | Compute next due date from last completion + interval |
| `calculateDaysUntilDue()` | Days between today and due date |
| `calculateUrgency()` | Map days to urgency level |
| `formatDaysUntilDue()` | Human-readable days ("3 days left") |
| `formatInterval()` | Human-readable interval ("Every 2 weeks") |
| `formatLastCompleted()` | Relative time ("3 hours ago") |

### Due Date Calculation Logic

```typescript
function calculateNextDueDate(
  lastCompletedAt: Date | null,
  intervalValue: number,
  intervalUnit: IntervalUnit,
  createdAt: Date
): Date {
  const baseDate = lastCompletedAt || createdAt;

  switch (intervalUnit) {
    case "days":
      return addDays(baseDate, intervalValue);
    case "weeks":
      return addWeeks(baseDate, intervalValue);
    case "months":
      return addMonths(baseDate, intervalValue);
  }
}
```

### Urgency Mapping

| Days Until Due | Urgency Level |
|----------------|---------------|
| < 0 | `overdue` |
| 0 | `today` |
| 1-7 | `this-week` |
| > 7 | `upcoming` |

---

## Deployment Workflows

### Initial Deployment (First Time)

```bash
# 1. Clone repository
git clone <repo-url>
cd days-to-thing-tracker

# 2. Run setup
python setup.py

# 3. Follow prompts:
#    - Enter Tailscale auth key
#    - Set hostname
#    - Wait for build and startup

# 4. Access via Tailscale URL
```

### Code Update Deployment

```bash
# 1. Pull latest changes
git pull

# 2. Run setup and choose option 3 (Rebuild)
python setup.py
# Select: 3

# 3. Restart containers
# Select: 2 (Refresh)
```

### Database Schema Changes

```bash
# 1. Update prisma/schema.prisma locally

# 2. Generate migration
npx prisma migrate dev --name your_migration_name

# 3. Run setup with option 4 (Full)
python setup.py
# Select: 4
```

### Troubleshooting Deployment

```bash
# View logs
python setup.py  # Select: 7

# Check container status
docker compose ps

# Reset Tailscale (fixes TLS issues)
python setup.py  # Select: r

# Clean rebuild
python setup.py  # Select: 5
```

---

## File Structure

```
days-to-thing-tracker/
├── src/
│   ├── app/
│   │   ├── api/
│   │   │   └── tasks/
│   │   │       ├── route.ts              # GET all, POST create
│   │   │       └── [id]/
│   │   │           ├── route.ts          # GET, PUT, DELETE single
│   │   │           ├── complete/
│   │   │           │   └── route.ts      # POST complete
│   │   │           └── history/
│   │   │               └── route.ts      # GET history
│   │   ├── layout.tsx                    # Root layout
│   │   ├── page.tsx                      # Home page (+ kiosk mode)
│   │   └── globals.css                   # Global styles
│   ├── components/
│   │   ├── ui/                           # Reusable UI components
│   │   ├── tasks/                        # Task-specific components
│   │   └── kiosk/                        # Kiosk mode components
│   ├── hooks/
│   │   ├── useTasks.ts                   # Task CRUD operations
│   │   ├── useCountdown.ts               # Auto-refresh timer
│   │   └── useKioskNavigation.ts         # Kiosk state machine
│   ├── lib/
│   │   ├── prisma.ts                     # Prisma client
│   │   └── date-utils.ts                 # Date calculations
│   └── types/
│       └── index.ts                      # TypeScript interfaces
├── prisma/
│   ├── schema.prisma                     # Database schema
│   └── migrations/                       # Migration files
├── docker/
│   └── Dockerfile                        # Multi-stage build
├── docker-compose.yml                    # Container orchestration
├── setup.py                              # Deployment manager
├── next.config.ts                        # Next.js configuration
├── prisma.config.ts                      # Prisma configuration
├── package.json                          # Dependencies
├── .env                                  # Environment variables (generated)
└── .docker/
    └── tailscale/
        └── state/                        # Tailscale credentials (persistent)
```

---

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker compose logs -f

# Common issues:
# - Missing TS_AUTHKEY: Run setup.py option 9
# - Port conflict: Check if port 3000 is in use
# - Build error: Run clean rebuild (option 5)
```

### Tailscale Connection Issues

```bash
# Check Tailscale status
docker exec days-tracker-tailscale tailscale status

# Common issues:
# - "not yet approved": Approve device in admin console
# - No IP assigned: Check auth key validity
# - TLS errors: Reset Tailscale (option r)
```

### Database Errors

```bash
# Check database file permissions
docker exec days-tracker-app ls -la /app/data/

# Run migrations manually
docker exec days-tracker-app node ./node_modules/prisma/build/index.js migrate deploy

# Common issues:
# - Migration failed: Check schema syntax
# - Permission denied: Rebuild container
```

### Website Not Loading

```bash
# Verify Tailscale Serve is running
docker exec days-tracker-tailscale tailscale serve status

# Check app is listening
docker exec days-tracker-tailscale curl -I http://127.0.0.1:3000

# Common issues:
# - Serve not configured: Run setup.py, option 9
# - App crashed: Check logs with option 7
```

### Data Loss Prevention

**Safe operations (data preserved):**
- Options 1-3 (Restart, Refresh, Rebuild)
- `docker compose down` (volumes preserved)

**Operations that may affect data:**
- Options 4-5 run migrations (usually safe, adds data)
- `docker compose down -v` (DELETES volumes!)
- Deleting `.docker/` directory (loses Tailscale state)

---

## Quick Reference

### Common Commands

```bash
# Start setup manager
python setup.py

# Quick restart
docker compose restart

# View logs
docker compose logs -f

# Check status
docker compose ps

# Enter container shell
docker exec -it days-tracker-app sh

# Check Tailscale
docker exec days-tracker-tailscale tailscale status
```

### Access URLs

- **Production**: `https://{hostname}.{tailnet}.ts.net`
- **API Base**: `https://{hostname}.{tailnet}.ts.net/api/tasks`
- **Kiosk Mode**: `https://{hostname}.{tailnet}.ts.net?kiosk=true`
