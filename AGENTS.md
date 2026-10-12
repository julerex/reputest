# Agent guide: Reputest

This file gives AI agents enough context to work effectively on the Reputest codebase.

## Project overview

**Reputest** is a Rust web service that:

- Monitors Twitter/X for **@reputest** mentions every 30 seconds and builds a directed social graph. Good vibes (`gmgv`) and megajoule transfers work with or without the hashtag, but the tweet must tag the bot.
- Processes **@reputest @username following?** mentions: fetches who that user follows and stores in `following` table.
- Processes **@reputest @username balance** mentions: replies with that user's megajoule balance (received minus sent).
- Exposes HTTP endpoints: Good Vibes dashboard, **Following**, **Megajoules** (static October 2026 US dollar prices per megajoule), health check, **web login** (X OAuth 2.0), and an **API playground** (call X API v2 paths with the logged-in user’s token).
- Uses a single **bot** token (from DB) for cronjobs and bot replies; **web sessions** (DB, encrypted) for login and playground.

## Tech stack

- **Rust** (edition 2021), **Axum** 0.7, **Tokio**
- **PostgreSQL** via **sqlx** 0.8 (async, `runtime-tokio-native-tls`)
- **Twitter/X API** v2: OAuth 2.0 PKCE, Bearer token, `api.twitter.com`
- **Security**: AES-256-GCM token encryption (`crypto.rs`), rate limiting (tower_governor), security headers (CSP, X-Frame-Options, etc.)

## Layout

| Path | Purpose |
|------|--------|
| `src/main.rs` | Entrypoint: routes, middleware, app state, server bind |
| `src/lib.rs` | Library root, re-exports |
| `src/config.rs` | Env config: port, BASE_URL/CALLBACK_URI (hardcoded), ALLOWED_USERNAME (hardcoded), TwitterConfig |
| `src/handlers.rs` | HTTP handlers: root, following, **megajoules**, health, reputest, **login**, **login/start**, **oauth callback** (at GET `/reputest`), **playground** GET/POST, logout; `AppState` |
| `src/megajoule_prices.rs` | Static October 2026 USD/MJ snapshot rows and conversion helpers for `GET /megajoules` |
| `src/oauth.rs` | OAuth: Bearer header, refresh token, PKCE helpers, auth URL, code exchange |
| `src/db.rs` | DB: tokens (encrypted), sessions (web login), good vibes, vibe scores |
| `src/crypto.rs` | Encrypt/decrypt tokens (AES-GCM), validate config |
| `src/twitter/` | X API: search, tweets, user lookup, `make_authenticated_request` (uses bot token) |
| `src/cronjob.rs` | @reputest mention cron (30s), vibe-request, following, megajoule, balance, and good-vibes handling |
| `sql/database_ddl.sql` | Schema: refresh_tokens, access_tokens, **sessions**, users, good_vibes, **following**, views, indexes |
| `scripts/` | Binaries: `authorize_bot`, `refresh_access_token`, `encrypt_token` |

## Conventions

- **Handlers**: Prefer **concrete return types** (e.g. `axum::response::Response`) when a handler has multiple response shapes (redirect vs HTML), to satisfy `Handler<_, _>` and avoid `impl IntoResponse` branch mismatches.
- **Extractors**: Use **`HeaderMap` + `Bytes`** for handlers that need cookies + body (e.g. playground). Using the full **`Request`** extractor can lead to `Handler` not being satisfied in this Axum version.
- **Security**: Never log tokens; use `html_escape()` for any user-derived HTML; validate paths (e.g. playground: only allow paths starting with `2/` for X API v2).
- **Config**: OAuth callback URI is **hardcoded** as `https://reputest.fly.dev/reputest`; allowed username is **hardcoded** as `julian_le_roux` (see `config.rs`).

## App state and routes

- **State**: `AppState { pool: PgPool, base_url, oauth_client_id, oauth_client_secret }` (all optional except pool). Built in `main.rs` from env (`get_base_url().ok()`, `XAPI_CLIENT_ID`, `XAPI_CLIENT_SECRET`).
- **Routes**: `/` (Good Vibes table), `/following` (following relationships table), `/megajoules` (static October 2026 USD per megajoule table; no database), `/reputest` (GET: “Reputesting!” or OAuth callback when `?code=&state=` present; POST: “Reputesting!”), `/health`, `/login`, `/login/start`, `/playground` (GET/POST), `/logout`. Nav on table pages: Good Vibes | Following | Megajoules.

## Database

- **Schema**: Apply `sql/database_ddl.sql` (includes `sessions` for web login). For existing DBs, run `sql/migrations/001_add_following.sql`.
- **Tokens**: `access_tokens` and `refresh_tokens` store **encrypted** values; use `crypto::encrypt_token` / `decrypt_token`; require `TOKEN_ENCRYPTION_KEY` (32-byte hex).
- **Sessions**: `sessions` table (id UUID, user_id, username, access_token, refresh_token encrypted, expires_at). Session ID is set in a cookie; validate and load via `db::get_session_by_id`.

## Running and testing

- **Run server**: `cargo run` (or `make` if target exists). Requires `DATABASE_URL`, `TOKEN_ENCRYPTION_KEY`; optional `PORT`, `RUST_LOG`, `XAPI_CLIENT_ID`, `XAPI_CLIENT_SECRET` for web login.
- **Tests**: `cargo test`. Many tests need `DATABASE_URL` and a real DB; they create `AppState` with `pool` and optional OAuth fields.
- **Bot auth**: `cargo run --bin authorize_bot` walks OAuth 2.0 and **prints** tokens. It does **not** write them to Postgres. Production reads encrypted rows from `access_tokens` / `refresh_tokens` using `TOKEN_ENCRYPTION_KEY`. To persist new tokens, encrypt with `encrypt_token` (same key as Fly) and insert, or save via `db::save_access_token` / refresh-token helpers.

### Cursor / cargo proxy error

When running `cargo` (build, test, fmt, clippy) inside Cursor you may see:

`error: unknown proxy name: 'Cursor-2.x-x86_64'; valid proxy names...`

The **Makefile** (see test/format/clippy targets) works around this by running **`unset ARGV0 && cargo ...`** before invoking cargo. Prefer **`make test`**, **`make format`**, **`make clippy`** when in Cursor, or run `unset ARGV0 && cargo <subcommand>` so cargo uses the real toolchain instead of the Cursor proxy. When validating changes (e.g. before committing), use **`make checkall`** to run tests, format, and clippy in one go.

## Gotchas

- **Mention-only ingestion.** The cron searches `@reputest` only. A bare `@alice gmgv` or `#megajoules` tweet never reaches the bot. `@reputest` itself is required; the `@` on the *target* username is optional (`@reputest goblinodds ?` works).
- **Command dispatch order** in `process_mentions`: following → balance → megajoule transfer → good vibes → vibe-score `?` → `vibecount`. First match wins. `reputest` is excluded as a good-vibes emitter, so `@reputest gmgv` does not record a vibe.
- **`gmgv` / `megajoules` hashtags are optional.** `@reputest @alice gmgv` and `@reputest 20 megajoules to @alice` match. Hashless `gmgv` requires a space before the word (`alicegmgv` does not match); `#gmgv` still allows zero spaces (`@alice#gmgv`).
- **Dedup.** Command replies (queries, following, balance, vibecount) are keyed in `vibe_requests`. Good vibes and megajoule transfers use their own tweet-id tables. Re-processing the same mention every 30 seconds is expected; skip, do not reply twice.
- **Cron expression is seven fields, seconds first.** `tokio-cron-scheduler` uses `"0/30 * * * * * *"` for every 30 seconds. A five-field crontab will not do what you think.
- **Token refresh.** Access tokens last about two hours. On HTTP 401 the client refreshes and retries. That immediate retry can return a Cloudflare 400; if the *next* 30-second cycle is 200, the token is fine. Treat refresh failure / invalid refresh token as the real auth break, not a single 400 after refresh.
- **Clippy `too_many_arguments`.** Limit is 7. Pass `&MentionedTweet` into record helpers instead of exploding tweet fields.
- **Deploy.** Push to `main` runs GitHub Actions **Fly Deploy** (`.github/workflows/fly-deploy.yml`). `fly.toml` has `min_machines_running = 1`; one Frankfurt machine started and one stopped is expected, not a failed deploy.
- **Older docs can lag.** `docs/BOT_SETUP.md`, `docs/TROUBLESHOOTING.md`, and `.env.example` may still mention hashtag searches every 5 minutes or `xapi_access_token` in the environment. The running service loads bot tokens from the database, not those environment variables.

## Useful references

- **README.md** — User-facing overview, quick start, API table, deployment.
- **docs/BOT_SETUP.md** — Twitter app and OAuth setup.
- **.env.example** — Env vars (no secrets).

When adding routes or handlers, keep the above in mind (state type, extractors, return types, and security).
