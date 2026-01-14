<div align="center">
  <b><h1>payme</h1></b>
</div>

<div align="center">
  <b> A very minimal personal finance tracking application. </b>
</div>

<p align="center">
  <a href="https://github.com/cachebag/payme/actions/workflows/ci.yml">
    <img src="https://github.com/cachebag/payme/actions/workflows/ci.yml/badge.svg" alt="CI">
  </a>
</p>

#

<img width="1532" height="1078" alt="image" src="https://github.com/user-attachments/assets/3981cde7-4e67-4fda-8fe8-ba965bb0a5ae" />

payme was designed for self-hosting in my homelab environment. Run it on a Raspberry Pi, NAS, or any always-on server to track your household finances privately without relying on third-party services. Your financial data stays on your network, under your control.

I grew tired of my spreadsheet, and did not care for any of the third party services out there. So I decided to build my own. As such, you can see this is very opinionated. The lack of advanced financial budgeting features is intentional, though, I am open to different features and components.

Generally, if you don't like it, fork it and make it your own or consider contributing to the project (read [CONTRIBUTING.md](CONTRIBUTING.md) for more information).

## Requirements

- Rust 1.75+
- Node.js 20+
- SQLite3

## Setup

### Backend

```bash
cd backend
cargo build --release
```

Environment variables:
See `.env.example` for all available variables.

```bash 
DATABASE_URL=sqlite:payme.db?mode=rwc
JWT_SECRET=some-random-string
PORT=3001
``` 


## Running both services

The `run.sh` script starts both the backend and frontend simultaneously:

```bash
chmod +x run.sh
./run.sh
```

This launches:
- Backend at http://localhost:3001
- Frontend at http://localhost:3000

Press `Ctrl+C` to stop both services.

You can obviously run the backend and frontend separately if you want to by navigating to the respective directories and running the commands there.

## Database

SQLite database created at `backend/payme.db`. Tables auto-migrate on startup.

Export/import database via the UI download button or `/api/export` endpoint.

## OpenAPI Swagger endpoint

To view all the api endpoints and schemas, go to: http://localhost:3001/swagger-ui

## Docker

Docker is the recommended way to deploy payme in a homelab. The multi-stage build creates a minimal image with just the compiled binary and static frontend assets.

### Using Docker Compose (Recommended)

```bash
# Set a secure JWT secret
echo "JWT_SECRET=$(openssl rand -base64 32)" > .env

# Build and start
docker compose up -d
```

Access payme at http://your-ip:3001

### Manual Docker Build

```bash
docker build -t payme .
docker run -d \
  --name payme \
  -p 3001:3001 \
  -v payme_data:/data \
  -e JWT_SECRET=your-secret-key \
  payme
```

### Data Persistence

The SQLite database is stored in a Docker volume at `/data`. To backup:

```bash
docker cp payme:/data/payme.db ./backup.db
```

### Reverse Proxy

For production, place payme behind a reverse proxy (nginx, Caddy, Traefik) with HTTPS. Example nginx config:

```nginx
server {
    listen 443 ssl;
    server_name finance.yourdomain.com;

    location / {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```
