# Soliloquio Public Blog — Starter Template

A minimal read-only blog frontend for [Soliloquio](../../README.md)'s public API. Fork this and customize it for your own blog.

## Prerequisites

- [Deno](https://deno.land/manual/getting_started/installation)
- A running Soliloquio backend
- An API key created from the Soliloquio admin UI (Settings → API Keys)

## Setup

```bash
cp .env.example .env
# edit .env: set PUBLIC_API_URL, PUBLIC_API_KEY, SITE_TITLE
deno task start
```

Open http://localhost:3000.

## Env vars

| Variable | Required | Default | Description |
|---|---|---|---|
| `PUBLIC_API_URL` | yes | — | Backend `/public` endpoint, e.g. `https://example.com/public` |
| `PUBLIC_API_KEY` | yes | — | API key from the admin UI (`slq_...`) |
| `SITE_TITLE` | yes | — | Blog title shown in `<title>` and OG tags |
| `SITE_URL` | no | — | Canonical base URL, used for `og:url` |
| `POSTS_PER_PAGE` | no | `10` | Posts per page (max 100) |
| `PORT` | no | `3000` | Dev server port |

## Commands

```bash
deno task start    # dev server with hot reload
deno task build    # production build
deno task preview  # preview production build
deno task check    # lint + type check
```

## Customization

- **List layout**: edit `components/PostCard.tsx`
- **Post layout**: edit `routes/posts/[slug].tsx`; content is rendered into `<div class="prose">` — customize via [Tailwind typography](https://tailwindcss.com/docs/typography-plugin) in `tailwind.config.ts`
- **Site title**: edit `components/SiteTitle.tsx`
- **Pagination**: edit `components/Pagination.tsx`

## Deployment

This is a standard Fresh app. Deploy to [Deno Deploy](https://deno.com/deploy) or run with Docker.

For Docker, use the same pattern as `client/Dockerfile` in the main repo, swapping the app directory.
