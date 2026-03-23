# Soliloquio

Self-hosted blogging platform with a public GraphQL API.

Write and publish posts in markdown. Expose them through a public API so any frontend can consume your blog. Run it yourself.

## Features

- Markdown post editor with live preview
- Public GraphQL API with API key authentication and per-key rate limiting
- Media library with automatic WebP conversion
- Email verification and password reset
- Single-user mode (locks registration after first account)
- JWT auth with multi-device refresh tokens
- GraphQL subscriptions

## Quick start

**Prerequisites:** [Rust](https://rustup.rs), [Docker](https://docs.docker.com/get-docker/), [Deno](https://deno.land/manual/getting_started/installation)

```sh
# 1. Start PostgreSQL + MailCatcher
docker-compose up -d

# 2. Configure environment
cp .env.example .env
# Edit .env — at minimum set TOKEN_SECRET and HOST_NAME

# 3. Set up the database
make reset_database

# 4. Start the backend (http://localhost:8000)
cargo run

# 5. Start the frontend (http://localhost:3000)
cd client && deno task start
```

GraphiQL IDE is available at http://localhost:8000.

## Production deployment

```sh
cp .env.example .env
# Edit .env for production values (see Environment variables below)
docker-compose -f compose.prod.yml up -d
```

Key production settings:
- `SECURE_COOKIES=true`
- `BIND_ADDR=0.0.0.0:8000`
- `LOG_FORMAT=json`
- Strong random `TOKEN_SECRET`

## Environment variables

### Database

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | PostgreSQL connection string |
| `DB_NAME` | `soliloquio` | Database name |
| `POSTGRES_PASSWORD` | — | PostgreSQL password |

### Auth

| Variable | Default | Description |
|---|---|---|
| `TOKEN_SECRET` | — | JWT signing secret |
| `TOKEN_EXPIRATION_SECONDS` | `3600` | Access token lifetime (seconds) |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | `7` | Refresh token lifetime (days) |
| `HOST_NAME` | `localhost` | JWT issuer hostname |

### Server

| Variable | Default | Description |
|---|---|---|
| `BIND_ADDR` | `127.0.0.1:8000` | Server bind address |
| `SECURE_COOKIES` | `false` | Set `true` in production |
| `SINGLE_USER_MODE` | `false` | Disable registration after first user |
| `RUST_LOG` | `info` | Log level filter |
| `LOG_FORMAT` | `pretty` | `json` for structured logging |

### CORS

| Variable | Default | Description |
|---|---|---|
| `ALLOWED_ORIGINS` | `http://localhost:3000` | Comma-separated allowed origins (main endpoint) |
| `PUBLIC_CORS_ORIGINS` | `*` | Comma-separated allowed origins (`/public` endpoint) |

### Email

Required for email verification and password reset.

| Variable | Default | Description |
|---|---|---|
| `SMTP_HOST` | `localhost` | SMTP server |
| `SMTP_PORT` | `1025` | SMTP port |
| `SMTP_USER` | — | SMTP username |
| `SMTP_PASSWORD` | — | SMTP password |
| `SMTP_FROM` | `noreply@example.com` | From address |
| `APP_BASE_URL` | `http://localhost:3000` | Frontend base URL for email links |

### Public API rate limiting

| Variable | Default | Description |
|---|---|---|
| `PUBLIC_MAX_COMPLEXITY` | `1000` | Max query complexity per request |
| `PUBLIC_MAX_DEPTH` | `5` | Max query depth per request |
| `PUBLIC_COMPLEXITY_BUDGET` | `10000` | Complexity budget per window per API key |
| `PUBLIC_COMPLEXITY_WINDOW_SECS` | `60` | Budget window duration (seconds) |

### Assets

| Variable | Default | Description |
|---|---|---|
| `UPLOAD_DIR` | `./uploads` | Local directory for uploaded assets |

## Architecture

**Backend:** Rust workspace — `models` (SeaORM entities) → `services` (auth, email, assets) → `graphql` (async-graphql schema) → `main` (Actix-web server).

**Frontend:** Deno/Fresh with Preact, React Query, and Tailwind CSS. Uses a BFF pattern — all GraphQL calls go through `/api/graphql` so auth cookies stay server-side. See [`client/README.md`](client/README.md).

### Endpoints

| Path | Description |
|---|---|
| `POST /` | Authenticated GraphQL API |
| `GET /` | GraphiQL IDE |
| `WS /ws` | GraphQL subscriptions |
| `POST /public` | Public API (API key auth) |
| `POST /upload` | Asset upload |
| `GET /assets/{key}` | Asset retrieval |
| `GET /health` | Health check |

## License

[AGPL-3.0](LICENSE) — forks that run as a service must open-source their changes.
