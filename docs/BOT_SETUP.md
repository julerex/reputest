# Twitter Bot Setup Guide

Complete guide for configuring OAuth 2.0 User Context authentication with the Twitter/X Developer Portal.

> **Quick Start**: If you just need the commands, run `cargo run --bin authorize_bot` and follow the prompts.

---

## Prerequisites

- A Twitter/X account
- [Twitter Developer Account](https://developer.twitter.com/) (apply if you don't have one)
- Rust toolchain installed

---

## Step 1: Create a Twitter App

### 1.1 Access the Developer Portal

1. Go to [developer.twitter.com](https://developer.twitter.com/)
2. Sign in with your Twitter account
3. Navigate to **Projects & Apps** → Your Project

### 1.2 Create or Configure Your App

1. Click **"+ Add App"** or select your existing app
2. Click the **gear icon** to open App Settings
3. Go to the **Settings** tab

### 1.3 Configure User Authentication

1. Find **"User authentication settings"** section
2. Click **"Set up"** or **"Edit"**
3. Configure the following:

| Setting | Value |
|---------|-------|
| **OAuth 2.0** | ✅ Enabled |
| **Type of App** | `Web App, Automated App or Bot` |
| **Callback URI** | `http://localhost:8080/callback` |
| **Website URL** | `https://reputest.fly.dev` (or your domain) |

4. **Required Scopes** — Select these under "App permissions":
   - `tweet.read` — Read tweets
   - `tweet.write` — Post and delete tweets
   - `users.read` — Read user profile info
   - `offline.access` — Get refresh tokens

5. Click **Save**

### 1.4 Get Your Credentials

After saving, note down your:
- **Client ID** (public identifier)
- **Client Secret** (keep this secure!)

---

## Step 2: Run the Authorization Flow

The built-in script handles the OAuth 2.0 PKCE flow:

```bash
cargo run --bin authorize_bot
```

The script will:

1. **Prompt for credentials** — Enter your Client ID and Client Secret
2. **Generate auth URL** — A secure authorization URL with PKCE challenge
3. **Open browser** — Authorize the app with your Twitter account
4. **Exchange code** — Automatically exchange the auth code for tokens
5. **Store encrypted** — Save tokens to the database (encrypted with AES-256-GCM)

### Expected Output

```
🔐 Twitter Bot Authorization
Enter your Client ID: abc123...
Enter your Client Secret: ***
Enter Redirect URI [http://localhost:8080/callback]: 

Opening browser for authorization...
Waiting for callback...

✅ Authorization successful!
Access token stored in database (encrypted)
Refresh token stored in database (encrypted)
```

---

## Step 3: Token Management

### How Tokens Work

| Token | Lifespan | Purpose |
|-------|----------|---------|
| **Access Token** | ~2 hours | Authenticates API requests |
| **Refresh Token** | ~6 months | Obtains new access tokens |

### Automatic Refresh

The service automatically refreshes expired tokens:

1. Makes API request
2. Receives 401 Unauthorized
3. Uses refresh token to get new access token
4. Retries the original request
5. Saves new tokens to database

### Manual Refresh

If automatic refresh fails:

```bash
cargo run --bin refresh_token
```

### Re-Authorization

If your refresh token expires (after ~6 months of inactivity):

```bash
cargo run --bin authorize_bot
```

---

## Step 4: Verify Setup

### Check Database Tokens

```sql
-- Check if tokens exist (don't display actual values!)
SELECT id, LENGTH(token) as token_length, created_at 
FROM access_tokens 
ORDER BY created_at DESC 
LIMIT 1;
```

### Test the Bot

Start the server and check logs:

```bash
RUST_LOG=info cargo run
```

The bot should start monitoring for #gmgv hashtags within 5 minutes.

---

## Scope Reference

| Scope | Permission | Used For |
|-------|-----------|----------|
| `tweet.read` | Read tweets | Searching #gmgv hashtags |
| `tweet.write` | Post tweets | Replying to vibe queries |
| `users.read` | Read profiles | Getting user info |
| `offline.access` | Refresh tokens | Automatic token renewal |

---

## Security Checklist

- [ ] Never commit Client Secret to version control
- [ ] Store `TOKEN_ENCRYPTION_KEY` securely (Fly.io secrets, etc.)
- [ ] Tokens are encrypted at rest with AES-256-GCM
- [ ] Rotate credentials if compromised
- [ ] Monitor usage in Developer Portal

---

## Common Issues

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid client` | Wrong Client ID/Secret | Verify credentials in Developer Portal |
| `Invalid redirect_uri` | URI mismatch | Must match exactly in app settings |
| `Invalid scope` | Missing permissions | Enable required scopes in app settings |
| `Token expired` | Access token expired | Should auto-refresh; run `refresh_token` if not |
| `Refresh token invalid` | Token revoked/expired | Re-run `authorize_bot` |

---

## Additional Resources

- [Twitter OAuth 2.0 Documentation](https://developer.twitter.com/en/docs/authentication/oauth-2-0)
- [Twitter API v2 Reference](https://developer.twitter.com/en/docs/twitter-api)
- [Rate Limits](https://developer.twitter.com/en/docs/twitter-api/rate-limits)
