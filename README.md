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
- Bun 1.3+
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
PAYME_BIND=127.0.0.1:3001
PAYME_STATIC_DIR=frontend/dist
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

To view all the api endpoints and schemas, go to: http://your-ip/swagger-ui

## Rafael deployment

In the Rafael monorepo, payme runs as a user-level systemd service:

```bash
mkdir -p ~/rafael/data/payme
printf 'JWT_SECRET=%s\n' "$(openssl rand -base64 32)" > ~/rafael/data/payme/payme.env
chmod 600 ~/rafael/data/payme/payme.env

cd ~/rafael/services/payme/frontend
bun install --frozen-lockfile
bun run build

cd ~/rafael
cargo build --release -p payme

mkdir -p ~/.config/systemd/user
ln -sf ~/rafael/infra/systemd/rafael-payme.service ~/.config/systemd/user/rafael-payme.service
systemctl --user daemon-reload
systemctl --user enable --now rafael-payme.service
```

The service binds to `127.0.0.1:3032` by default. Rafael exposes it publicly
through Tailscale Funnel on:

```txt
https://rafael.taild0efc0.ts.net:8443/
```

The backend auth cookie is named `payme_token` so it does not collide with
other apps on the same Tailscale hostname.
