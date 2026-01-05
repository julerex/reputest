# Troubleshooting Guide

Common issues and solutions for Reputest.

---

## Quick Diagnostics

```bash
# Check server health
curl http://localhost:3000/health

# View logs with debug output
RUST_LOG=debug cargo run

# Check database connection
psql $DATABASE_URL -c "SELECT 1"

# Verify encryption key is set
echo $TOKEN_ENCRYPTION_KEY | wc -c  # Should be 65 (64 chars + newline)
```

---

## Startup Issues

### Server Won't Start

#### "TOKEN_ENCRYPTION_KEY is required"

```
SECURITY ERROR: Token encryption is not properly configured
Set TOKEN_ENCRYPTION_KEY environment variable with a 32-byte hex key.
```

**Solution**: Generate and set an encryption key:
```bash
export TOKEN_ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

#### "DATABASE_URL environment variable is not set"

**Solution**: Set your PostgreSQL connection string:
```bash
export DATABASE_URL="postgres://user:password@localhost/reputest"
```

#### "Failed to create database pool"

**Causes**:
- PostgreSQL not running
- Wrong credentials
- Database doesn't exist

**Solutions**:
```bash
# Check PostgreSQL is running
pg_isready

# Create database if missing
createdb reputest

# Test connection
psql $DATABASE_URL -c "SELECT 1"
```

#### Port Already in Use

```
Error: Address already in use (os error 98)
```

**Solution**: Use a different port or kill the existing process:
```bash
export PORT=8080
# or
lsof -i :3000 | grep LISTEN | awk '{print $2}' | xargs kill
```

---

## Twitter API Issues

### 401 Unauthorized

**Symptoms**: API calls fail with status 401

**Common Causes**:

| Cause | Solution |
|-------|----------|
| Expired access token | Should auto-refresh; check logs |
| Missing refresh token | Re-run `cargo run --bin authorize_bot` |
| Revoked app permissions | Check Developer Portal settings |
| Wrong token in database | Re-authorize the bot |

**Debugging**:
```bash
# Check token exists
psql $DATABASE_URL -c "SELECT id, created_at FROM access_tokens ORDER BY created_at DESC LIMIT 1"

# Run with debug logs
RUST_LOG=debug cargo run
```

### 403 Forbidden

**Causes**:
- App doesn't have required permissions
- User revoked app access

**Solution**: Verify app permissions in [Developer Portal](https://developer.twitter.com/):
- `tweet.read` ✅
- `tweet.write` ✅
- `users.read` ✅
- `offline.access` ✅

### 429 Rate Limited

**Symptoms**: `Rate limit exceeded` errors

**Twitter Rate Limits**:
| Endpoint | Limit |
|----------|-------|
| Search tweets | 450/15 min |
| Post tweet | 300/3 hours |
| User lookup | 100/15 min |

**Solutions**:
- Wait for rate limit reset (check `x-rate-limit-reset` header)
- The cronjob runs every 5 minutes by default, which should stay within limits

### "Invalid redirect_uri"

**Solution**: Ensure the redirect URI in your code matches **exactly** what's configured in the Developer Portal, including:
- Protocol (`http://` vs `https://`)
- Port number
- Path (`/callback`)

---

## Database Issues

### "relation does not exist"

```
ERROR: relation "good_vibes" does not exist
```

**Solution**: Run the schema migrations:
```bash
psql $DATABASE_URL -f sql/database_ddl.sql
```

### "Decryption failed - wrong key"

**Cause**: Encryption key changed after tokens were stored

**Solution**: Re-authorize with the current key:
```bash
cargo run --bin authorize_bot
```

### Duplicate Key Violations

```
ERROR: duplicate key value violates unique constraint
```

This is expected behavior when the same good vibes relationship is detected twice. The service handles this gracefully.

---

## Cronjob Issues

### Cronjob Not Running

**Check logs for**:
```
Starting GMGV hashtag monitoring cronjob
```

If missing, check for scheduler errors:
```bash
RUST_LOG=debug cargo run 2>&1 | grep -i cron
```

### No Tweets Found

**Possible causes**:
1. No recent #gmgv tweets exist
2. Twitter API credentials issue
3. Search query returning empty results

**Debug**:
```bash
RUST_LOG=debug cargo run
# Look for: "Starting scheduled search for #gmgv tweets"
# And: "Found X tweets with #gmgv hashtag"
```

---

## Deployment Issues (Fly.io)

### Secrets Not Set

```bash
# List current secrets
fly secrets list

# Set required secrets
fly secrets set DATABASE_URL="postgres://..."
fly secrets set TOKEN_ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

### Machine Not Starting

```bash
# Check machine status
fly status

# View recent logs
fly logs

# Force restart
fly machine restart
```

### Connection Refused

**Check**:
1. App is running: `fly status`
2. Port is correct in `fly.toml`: `internal_port = 8080`
3. Health check is passing

---

## Log Analysis

### Enable Debug Logging

```bash
# Local development
RUST_LOG=debug cargo run

# Fly.io
fly secrets set RUST_LOG=debug
fly deploy
```

### Key Log Messages

| Message | Meaning |
|---------|---------|
| `Database pool created successfully` | DB connection OK |
| `Security configuration validated` | Encryption key OK |
| `Starting scheduled search for #gmgv` | Cronjob running |
| `Found N tweets with #gmgv hashtag` | Search working |
| `Received response with status: 401` | Token issue |
| `Token refreshed successfully` | Auto-refresh worked |

### Viewing Fly.io Logs

```bash
# Stream live logs
fly logs

# Recent logs
fly logs --no-tail

# Filter by level
fly logs | grep ERROR
```

---

## Getting Help

1. **Check this guide** for common issues
2. **Enable debug logging** to see detailed errors
3. **Review Twitter API status**: [api.twitterstat.us](https://api.twitterstat.us/)
4. **Check Fly.io status**: [status.flyio.net](https://status.flyio.net/)

If you're still stuck, open an issue with:
- Error message (full text)
- Relevant log output (sanitize tokens!)
- Steps to reproduce

