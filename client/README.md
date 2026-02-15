# Soliloquio Client

Deno/Fresh frontend with Preact, React Query, and Tailwind CSS.

## Development Commands

```bash
deno task start      # Dev server with hot reload
deno task build      # Production build
deno task preview    # Preview production build
deno task check      # Lint + type check
deno task manifest   # Regenerate fresh.gen.ts
```

Requires Deno: https://deno.land/manual/getting_started/installation

## Environment Variables

| Variable           | Default                         | Description         |
| ------------------ | ------------------------------- | ------------------- |
| `PORT`             | 3000                            | Dev server port     |
| `GRAPHQL_ENDPOINT` | `http://localhost:8000/graphql` | Backend GraphQL URL |

## Architecture

### Directory Structure

```
client/
├── routes/           # File-based routing (SSR pages)
│   ├── _app.tsx      # Root layout
│   ├── index.tsx     # Home (posts list)
│   ├── api/          # BFF endpoints
│   │   ├── graphql.ts    # GraphQL proxy with auth
│   │   └── auth/         # Auth endpoints (me, logout, refresh)
│   ├── auth/         # signin, signup, change_password
│   └── posts/        # new, [id], [id]/edit
├── islands/          # Hydrated interactive components
├── components/       # Pure presentational components
├── services/         # API layer (auth/, posts/)
│   └── {domain}/
│       ├── api.ts    # GraphQL operations
│       ├── hooks.ts  # React Query hooks
│       ├── keys.ts   # Query key factory
│       └── types.ts  # TypeScript interfaces
├── domains/          # Shared type definitions
├── utils/            # graphql_client, query_client
└── static/           # Static assets + Tailwind entry
```

### Component Hierarchy

```
Routes (SSR) → Islands (Hydrated) → Components (Presentational)
```

- **Routes**: Server-rendered pages, minimal interactivity
- **Islands**: Client-hydrated components wrapped in `QueryProvider`
- **Components**: Reusable UI elements (Button, Input, Card)

### State Management

| Scope        | Solution                              |
| ------------ | ------------------------------------- |
| Server state | React Query (`@tanstack/react-query`) |
| Local state  | Preact hooks (`useState`)             |
| Auth state   | httpOnly cookies (server-managed)     |

## BFF (Backend For Frontend)

The client uses a BFF pattern where all GraphQL requests go through
`/api/graphql` instead of directly to the backend. This enables secure
cookie-based authentication.

### GraphQL Proxy (`/api/graphql`)

- Forwards browser cookies to backend
- Forwards `Set-Cookie` headers from backend to browser
- Auto-retries on 401: calls refresh endpoint, retries original request
- Returns 401 to client on refresh failure (triggers redirect to signin)

### Auth Endpoints

| Endpoint            | Method | Description                    |
| ------------------- | ------ | ------------------------------ |
| `/api/auth/me`      | GET    | Check auth state, returns user |
| `/api/auth/logout`  | POST   | Revoke token + clear cookies   |
| `/api/auth/refresh` | POST   | Manual token refresh           |

## Authentication

Uses httpOnly cookies for secure token storage. Tokens are never exposed to
JavaScript.

### Flow

1. **Sign in/up**: Backend sets `access_token` and `refresh_token` cookies
2. **API requests**: BFF proxy forwards cookies to backend
3. **Token expiry**: BFF auto-refreshes on 401, retries request
4. **Logout**: BFF calls backend logout + clears cookies

### Cookies

| Cookie          | Duration | Purpose          |
| --------------- | -------- | ---------------- |
| `access_token`  | 1 hour   | JWT for API auth |
| `refresh_token` | 7 days   | Token refresh    |

Both are `httpOnly`, `SameSite=Lax`, `Path=/`.

## API Layer

GraphQL client with cookie-based auth:

- All requests include `credentials: "include"`
- No token management in client code
- Queries/mutations in `services/{domain}/api.ts`
- React Query hooks in `services/{domain}/hooks.ts`

## Key Dependencies

| Purpose       | Package                       |
| ------------- | ----------------------------- |
| Framework     | Fresh 1.7.3                   |
| UI            | Preact 10.22.0                |
| Data fetching | @tanstack/react-query 5.17.15 |
| GraphQL       | graphql-request 6.1.0         |
| Styling       | Tailwind CSS 3.4.1            |
