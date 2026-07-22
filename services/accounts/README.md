# PugdyMon Accounts API

Email/password auth + profile for Nest / web.

## Quick start (Docker)

```bash
cd services/accounts
cp .env.example .env
# edit POSTGRES_PASSWORD and JWT_SECRET
docker compose up --build
```

- API: `http://127.0.0.1:8788`
- Health: `GET /health`
- Postgres (host): `127.0.0.1:5434`
- Public (free tunnel): `https://pudgymon-api.boing.network`

## Local cargo run

```bash
docker compose up -d db
export DATABASE_URL=postgres://pudgymon:$POSTGRES_PASSWORD@127.0.0.1:5434/pudgymon_accounts
export JWT_SECRET=your-secret
cargo run -p pudgymon-accounts
```

## Endpoints

| Method | Path | Body |
|--------|------|------|
| POST | `/v1/auth/signup` | `{ email, password, display_name }` |
| POST | `/v1/auth/login` | `{ email, password }` |
| GET | `/v1/me` | Bearer token |
| PATCH | `/v1/me` | `{ display_name?, boing_wallet? }` |

Signup/login return `{ access_token, profile }`.
