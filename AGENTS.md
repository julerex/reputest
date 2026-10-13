# Agent guide: Reputest

This file gives AI agents enough context to work effectively on the Reputest codebase.

## Project overview

**Reputest** is a Rust web service that:

- Monitors Twitter/X for **@reputest** mentions every 30 seconds and builds a directed social graph. Good vibes (`gmgv`) and megajoule transfers work with or without the hashtag, but the tweet must tag the bot.
- Processes **@reputest @username following?** mentions: fetches who that user follows and stores in `following` table.
- Processes **@reputest @username balance** mentions: replies with that user's megajoule balance (received minus sent).
- Exposes HTTP endpoints: Good Vibes dashboard, **Following**, **Megajoules** (static October 2026 US dollar prices per megajoule), health check, **web login** (X OAuth 2.0), and an **API playground** (call X API v2 paths with the logged-in user’s token).
- Uses a single **bot** token (from DB) for cronjobs and bot replies; **web sessions** (DB, encrypted) for login and playground.
- Stores **megajoule transfers and balances** in a **TigerBeetle** cluster (one replica on Fly). PostgreSQL keeps sessions, tokens, following, good vibes, and the historical `megajoule` table used for the one-shot import.

## Tech stack

- **Rust** (edition 2021), **Axum** 0.7, **Tokio**
- **PostgreSQL** via **sqlx** 0.8 (async, `runtime-tokio-native-tls`)
- **Twitter/X API** v2: OAuth 2.0 PKCE, Bearer token, `api.twitter.com`
- **TigerBeetle** 0.17.9 for the megajoule ledger (in-tree Rust client, not crates.io)
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
| `src/ledger.rs` | TigerBeetle megajoule ledger: account/transfer ids, balance, record transfer |
| `src/twitter/` | X API: search, tweets, user lookup, `make_authenticated_request` (uses bot token) |
| `src/cronjob.rs` | @reputest mention cron (30s), vibe-request, following, megajoule, balance, and good-vibes handling |
| `deploy/tigerbeetle/` | Single-replica TigerBeetle Fly app (Dockerfile, start script, fly.toml) |
| `sql/database_ddl.sql` | Schema: refresh_tokens, access_tokens, **sessions**, users, good_vibes, **following**, views, indexes |
| `scripts/` | Binaries: `authorize_bot`, `refresh_access_token`, `encrypt_token`, `import_megajoules`. Helpers: `build_tigerbeetle_client.sh`, `tigerbeetle_dev.sh` |

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
- **Megajoule ledger**: Twitter user ids are TigerBeetle account ids; tweet ids are transfer ids (replay is idempotent). Ledger `1`, code `1`. A transfer debits the sender and credits the receiver. Balance is `credits_posted - debits_posted`. Account flags stay unset, so a sender can go negative. Amount `0` and self-transfers are rejected. The PostgreSQL `megajoule` table is historical; live writes go through `src/ledger.rs`.

## Running and testing

- **Run server**: `cargo run` (or `make` if target exists). Requires `DATABASE_URL`, `TOKEN_ENCRYPTION_KEY`, `TB_CLUSTER_ID`, `TB_ADDRESS`; optional `PORT`, `RUST_LOG`, `XAPI_CLIENT_ID`, `XAPI_CLIENT_SECRET` for web login. For a local replica, run `./scripts/tigerbeetle_dev.sh` first (default cluster id `1`, address `127.0.0.1:3000`).
- **TigerBeetle client**: `./scripts/build_tigerbeetle_client.sh` clones release 0.17.9 into gitignored `third_party/tigerbeetle` and builds the Rust client assets. `make test` / `make clippy` run this first.
- **Tests**: `cargo test`. Many tests need `DATABASE_URL` and a real DB; they create `AppState` with `pool` and optional OAuth fields. Ledger unit tests do not need a running cluster.
- **Bot auth**: `cargo run --bin authorize_bot` walks OAuth 2.0 and **prints** tokens. It does **not** write them to Postgres. Production reads encrypted rows from `access_tokens` / `refresh_tokens` using `TOKEN_ENCRYPTION_KEY`. To persist new tokens, encrypt with `encrypt_token` (same key as Fly) and insert, or save via `db::save_access_token` / refresh-token helpers.

### Cursor / cargo proxy error

When running `cargo` (build, test, fmt, clippy) inside Cursor you may see:

`error: unknown proxy name: 'Cursor-2.x-x86_64'; valid proxy names...`

The **Makefile** (see test/format/clippy targets) works around this by running **`unset ARGV0 && cargo ...`** before invoking cargo. Prefer **`make test`**, **`make format`**, **`make clippy`** when in Cursor, or run `unset ARGV0 && cargo <subcommand>` so cargo uses the real toolchain instead of the Cursor proxy. When validating changes (e.g. before committing), use **`make checkall`** to run tests, format, and clippy in one go.

## Gotchas

- **Mention-only ingestion.** The cron searches `@reputest` only. A bare `@alice gmgv` or `#megajoules` tweet never reaches the bot. `@reputest` itself is required; the `@` on the *target* username is optional (`@reputest goblinodds ?` works).
- **Command dispatch order** in `process_mentions`: following → balance → megajoule transfer → good vibes → vibe-score `?` → `vibecount`. First match wins. `reputest` is excluded as a good-vibes emitter, so `@reputest gmgv` does not record a vibe.
- **`gmgv` / `megajoules` hashtags are optional.** `@reputest @alice gmgv` and `@reputest 20 megajoules to @alice` match. Hashless `gmgv` requires a space before the word (`alicegmgv` does not match); `#gmgv` still allows zero spaces (`@alice#gmgv`).
- **Dedup.** Command replies (queries, following, balance, vibecount) are keyed in `vibe_requests`. Good vibes use the `good_vibes` tweet-id table. Megajoule transfers use the TigerBeetle transfer id (the tweet id). Re-processing the same mention every 30 seconds is expected; skip, do not reply twice.
- **Cron expression is seven fields, seconds first.** `tokio-cron-scheduler` uses `"0/30 * * * * * *"` for every 30 seconds. A five-field crontab will not do what you think.
- **Token refresh.** Access tokens last about two hours. On HTTP 401 the client refreshes and retries. That immediate retry can return a Cloudflare 400; if the *next* 30-second cycle is 200, the token is fine. Treat refresh failure / invalid refresh token as the real auth break, not a single 400 after refresh.
- **TigerBeetle client release.** `zig build clients:rust -Drelease` without `-Dconfig-release=0.17.9 -Dconfig-release-client-min=0.16.4` embeds client version `65535.0.0`. The replica then evicts the session as too new. `scripts/build_tigerbeetle_client.sh` passes those flags.
- **TigerBeetle clone needs full git history.** A `--depth 1` clone fails the Zig client build with `malformed object name HEAD^` (`git tag --merged HEAD^`). Clone the `0.17.9` tag with history. The official Rust crate is not on crates.io; Cargo uses a path dependency on gitignored `third_party/tigerbeetle/src/clients/rust`, whose `build.rs` needs `assets/tb_client.h` and `assets/lib/*/libtb_client.a` from that Zig build. Enable `overflow-checks` in this crate’s Cargo profiles.
- **TigerBeetle addresses are IP only.** `Client::new` and `tigerbeetle start --addresses` reject hostnames. Resolve `reputest-tigerbeetle.internal:3000` to `[fdaa:…]:3000` in `ledger.rs` before connecting. The replica start script reads `fly-local-6pn` from `/etc/hosts`. Cluster id `0` is reserved for tests.
- **Two Fly apps.** `.github/workflows/fly-deploy.yml` deploys only `reputest`. The replica is `reputest-tigerbeetle` (`deploy/tigerbeetle/`), deployed separately so a web deploy does not restart it. Do not add a public `[[services]]` / HTTP service: Fly will allocate public IPs, and the protocol is not HTTP. Keep the replica Machine with `fly machine update --autostop=off --restart always`. Replica count is fixed at format time. Re-import with `fly ssh console -a reputest -C /usr/local/bin/import_megajoules` (transfer ids make a second run a no-op).
- **Clippy `too_many_arguments`.** Limit is 7. Pass `&MentionedTweet` into record helpers instead of exploding tweet fields.
- **Deploy.** Push to `main` runs GitHub Actions **Fly Deploy** (`.github/workflows/fly-deploy.yml`). `fly.toml` has `min_machines_running = 1`; one Frankfurt machine started and one stopped is expected, not a failed deploy.
- **Check the live system when asked if something landed.** A commit on `main` is not evidence the bot, Postgres, or the ledger has the change. Use `fly status` and `fly logs -a reputest` for deploys, query Postgres or search git history for a removed table, and look up a megajoule transfer by tweet id on `reputest-tigerbeetle` (`fly proxy`, then `lookup_transfers`). Report what you found and the commands that reproduce the check.
- **Price a Fly change before deploying it.** For a new or resized always-on Machine, or for Managed Postgres, say what the current shape costs per month and what the smallest shape costs that still meets the dependency's own limits. Wait for an explicit yes before deploying a more expensive size. Frankfurt compute is about 1.154 times the Ashburn price. Volumes are $0.15 per GB and do not get that multiplier. Managed Postgres has no stop or scale-to-zero control; the plan bills until the cluster is deleted.
- **Older docs can lag.** `docs/BOT_SETUP.md`, `docs/TROUBLESHOOTING.md`, and `.env.example` may still mention hashtag searches every 5 minutes or `xapi_access_token` in the environment. The running service loads bot tokens from the database, not those environment variables.

## Useful references

- **README.md** — User-facing overview, quick start, API table, deployment.
- **docs/BOT_SETUP.md** — Twitter app and OAuth setup.
- **.env.example** — Env vars (no secrets).

When adding routes or handlers, keep the above in mind (state type, extractors, return types, and security).
