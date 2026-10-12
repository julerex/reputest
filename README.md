<p align="center">
  <img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/PostgreSQL-316192?style=for-the-badge&logo=postgresql&logoColor=white" alt="PostgreSQL">
  <img src="https://img.shields.io/badge/Twitter-1DA1F2?style=for-the-badge&logo=twitter&logoColor=white" alt="Twitter">
  <img src="https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge" alt="MIT License">
</p>

<h1 align="center">✨ Reputest</h1>

<p align="center">
  <strong>A social reputation graph built on good vibes</strong><br>
  Track positive relationships on Twitter/X and discover connection degrees between users
</p>

<p align="center">
  <a href="#-what-it-does">What It Does</a> •
  <a href="#-how-it-works">How It Works</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-api-reference">API</a> •
  <a href="#-deployment">Deployment</a>
</p>

---

## 🎯 What It Does

The [`reputest`](https://x.com/reputest) bot monitors Twitter/X for tweets that tag **@reputest** and builds a directed social graph of positive relationships. When someone tweets `@reputest @username gmgv` (the `#` on `gmgv` is optional), they're attesting to the good vibes they get from that X user, creating a connection in the reputation graph.

**Key Features:**

- 🔍 **Mention Monitoring** — Automatically scans for @reputest mentions every 30 seconds
- 📊 **Multi-Degree Analysis** — Calculates 1st through 4th degree connection paths
- 🤖 **Twitter Bot** — Users can query vibe scores by mentioning @reputest
- 🔐 **Encrypted Token Storage** — AES-256-GCM encryption for all OAuth tokens
- ⚡ **High Performance** — Built with Axum and async Rust for speed
- 🛡️ **Production Security** — Rate limiting, security headers, and XSS protection

## 🧠 How It Works

### The Good Vibes Graph

```
 Alice ──gmgv──▶ Bob ──gmgv──▶ Charlie ──gmgv──▶ Diana
   │                              │
   └──────────gmgv───────────────▶┘
```

*Arrows show attestations: who tweeted → who was mentioned*

- **Emitter**: The person emitting good vibes (mentioned user)
- **Sensor**: The person sensing/attesting to those vibes (author of the @reputest gmgv tweet)

### Degree Paths

When Alice queries her vibe score with Diana:

| Degree | Meaning | Example Path |
|--------|---------|--------------|
| **1st** | Direct connection | Alice → Diana |
| **2nd** | One intermediary | Alice → Bob → Diana |
| **3rd** | Two intermediaries | Alice → Bob → Charlie → Diana |
| **4th** | Three intermediaries | Alice → X → Y → Z → Diana |

### Query Your Vibes

Tweet `@reputest @username?` to get your vibe scores with that user:

```
@reputest @elonmusk?
```

Reply:
```
Your vibes for @elonmusk are:
1st degree: 0
2nd degree: 3
3rd degree: 12
```

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [PostgreSQL](https://www.postgresql.org/) 14+
- [Twitter Developer Account](https://developer.twitter.com/) with OAuth 2.0 app

### 1. Clone & Setup

```bash
git clone https://github.com/julerex/reputest.git
cd reputest
```

### 2. Database Setup

Create the database and run the schema:

```bash
createdb reputest
psql -d reputest -f sql/database_ddl.sql
```

### 3. Environment Variables

```bash
# Required
export DATABASE_URL="postgres://user:password@localhost/reputest"
export TOKEN_ENCRYPTION_KEY="$(openssl rand -hex 32)"  # 32-byte hex key

# Optional
export PORT=3000           # Default: 3000
export RUST_LOG=info       # Options: debug, info, warn, error
```

### 4. Twitter Bot Authorization

```bash
# Run the OAuth 2.0 authorization flow
cargo run --bin authorize_bot
```

This will guide you through:
1. Entering your Twitter OAuth 2.0 Client ID & Secret
2. Authorizing the app in your browser
3. Storing encrypted tokens in the database

### 5. Run

```bash
cargo run
```

Visit `http://localhost:3000` to see the Good Vibes dashboard.

## 📡 API Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/` | Good Vibes dashboard — displays all relationships with degree paths |
| `GET` | `/login` | Login page — "Login with X" to start OAuth 2.0 web flow |
| `GET` | `/login/start` | Starts OAuth flow (redirects to X) |
| `GET` | `/oauth/callback` | OAuth callback — exchanges code for session |
| `GET` | `/playground` | X API playground — type API path, see response (requires login) |
| `POST` | `/playground` | Submit X API request from playground |
| `GET` | `/logout` | Log out and clear session |
| `GET` | `/reputest` | Test endpoint — returns `"Reputesting!"` |
| `POST` | `/reputest` | Test endpoint — returns `"Reputesting!"` |
| `GET` | `/health` | Health check — returns `{"status": "healthy", "service": "reputest"}` |

### Dashboard

The homepage displays a comprehensive table showing all sensor-emitter pairs with their path counts across all four degrees:

| sensor | sensor name | emitter | emitter name | 1° | 2° | 3° | 4° |
|--------|-------------|---------|--------------|----|----|----|----|
| @alice | Alice Smith | @bob | Bob Jones | 1 | 0 | 0 | 0 |
| @alice | Alice Smith | @charlie | Charlie Brown | 0 | 2 | 5 | 8 |

### Web Login and API Playground

To use the X API playground with your own account:

1. Set `BASE_URL` (e.g. `http://localhost:3000`) and ensure `XAPI_CLIENT_ID` and `XAPI_CLIENT_SECRET` are set.
2. In the [X Developer Portal](https://developer.twitter.com/), set the Callback URI to **`https://reputest.fly.dev/reputest`** (hardcoded).
3. Visit `/login`, click "Login with X", authorize, then use `/playground` to call X API v2 paths (e.g. `2/users/me`) and see the response.
4. Optional: set `ALLOWED_USERNAME=julian_le_roux` to restrict login to that account only.

## ⚙️ Configuration

### Required Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `TOKEN_ENCRYPTION_KEY` | 32-byte hex key for AES-256-GCM encryption |

### Optional Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | HTTP server port |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |

### Generating an Encryption Key

```bash
# Generate a secure 32-byte key
openssl rand -hex 32
```

⚠️ **Security Note**: The server will refuse to start without a valid encryption key. All OAuth tokens are encrypted at rest.

## 🗄️ Database Schema

### Core Tables

```sql
-- Twitter users in the vibes graph
users (id, username, name, created_at)

-- Good vibes relationships (directed graph edges)
good_vibes (tweet_id, emitter_id, sensor_id, created_at)

-- OAuth tokens (encrypted)
access_tokens (id, token, created_at)
refresh_tokens (id, token, created_at)

-- Processed tweet tracking
vibe_requests (tweet_id)
```

### Pre-built Views

The schema includes optimized views for path counting:

- `view_good_vibes_degree_one` through `view_good_vibes_degree_four`
- `view_all_good_vibes_degrees` — Combined view used by the dashboard
- `view_easy_*` variants with human-readable usernames

## 📁 Project Structure

```
reputest/
├── src/
│   ├── main.rs          # Server initialization, routes, middleware
│   ├── config.rs        # Environment configuration
│   ├── handlers.rs      # HTTP route handlers
│   ├── db.rs            # Database operations & graph queries
│   ├── crypto.rs        # AES-256-GCM token encryption
│   ├── cronjob.rs       # Scheduled Twitter monitoring
│   ├── oauth.rs         # OAuth 2.0 token refresh
│   ├── twitter/
│   │   ├── mod.rs       # Twitter module exports
│   │   ├── api.rs       # API client & utilities
│   │   ├── search.rs    # Hashtag & mention search
│   │   ├── tweets.rs    # Tweet posting & replies
│   │   └── parsing.rs   # Tweet text parsing
│   ├── lib.rs           # Library exports
│   └── tests.rs         # Test suite
├── scripts/
│   ├── authorize_bot.rs      # OAuth 2.0 authorization flow
│   ├── refresh_access_token.rs  # Manual token refresh
│   └── encrypt_token.rs      # Token encryption utility
├── sql/
│   ├── database_ddl.sql      # Schema & views
│   └── database_init.sql     # Initial data (if any)
├── docs/
│   ├── BOT_SETUP.md          # Twitter OAuth 2.0 setup guide
│   ├── CLOUDFLARE_DOMAIN_SETUP.md  # Custom domain configuration
│   ├── TROUBLESHOOTING.md    # Common issues & solutions
│   └── DEBUGGING.md          # Developer debugging guide
├── Cargo.toml
├── Dockerfile
└── fly.toml              # Fly.io deployment config
```

## 🐳 Docker

```bash
# Build
docker build -t reputest .

# Run
docker run -p 3000:3000 \
  -e DATABASE_URL="postgres://..." \
  -e TOKEN_ENCRYPTION_KEY="$(openssl rand -hex 32)" \
  reputest
```

## ☁️ Deployment

### Fly.io

The project includes ready-to-use Fly.io configuration:

```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Deploy
fly launch  # First time
fly deploy  # Updates

# Set secrets
fly secrets set DATABASE_URL="postgres://..."
fly secrets set TOKEN_ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

The app is configured for:
- **Region**: Frankfurt (`fra`)
- **Memory**: 1GB
- **Port**: 8080 (internal)
- **HTTPS**: Forced
- **Minimum machines**: 1

## 🔧 Development

### Running Tests

```bash
cargo test                    # All tests
cargo test -- --nocapture     # With output
cargo test handlers           # Specific module
```

### Building for Release

```bash
cargo build --release
```

Release builds are optimized for size (`opt-level = "z"`) with LTO enabled.

### Utility Scripts

```bash
# Authorize bot (OAuth 2.0 flow)
cargo run --bin authorize_bot

# Manually refresh access token
cargo run --bin refresh_token

# Encrypt a token for database storage
cargo run --bin encrypt_token
```

## 🔒 Security

- **Token Encryption**: All OAuth tokens encrypted with AES-256-GCM
- **Rate Limiting**: 30 requests/minute per IP via `tower_governor`
- **Security Headers**: X-Content-Type-Options, X-Frame-Options, CSP, etc.
- **XSS Protection**: HTML escaping on all user-generated content
- **Input Validation**: Log sanitization to prevent injection attacks
- **Automatic Cleanup**: Old tokens purged after 24 hours

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Make your changes with tests
4. Run `cargo test` and `cargo clippy`
5. Commit (`git commit -m 'Add amazing feature'`)
6. Push (`git push origin feature/amazing`)
7. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) — Web framework
- [SQLx](https://github.com/launchbadge/sqlx) — Async PostgreSQL
- [tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler) — Job scheduling
- [tower-governor](https://github.com/benwis/tower-governor) — Rate limiting

---

<p align="center">
  <sub>Built with 🦀 and good vibes</sub>
</p>
