# Debugging Guide

Developer guide for debugging Reputest internals.

---

## Log Levels

| Level | Usage | Enable With |
|-------|-------|-------------|
| `error` | Failures only | `RUST_LOG=error` |
| `warn` | Warnings + errors | `RUST_LOG=warn` |
| `info` | Normal operation | `RUST_LOG=info` (default) |
| `debug` | Detailed debugging | `RUST_LOG=debug` |
| `trace` | Everything | `RUST_LOG=trace` |

### Per-Module Logging

```bash
# Debug only Twitter module
RUST_LOG=reputest::twitter=debug cargo run

# Debug database + info for everything else
RUST_LOG=info,reputest::db=debug cargo run

# Debug OAuth specifically
RUST_LOG=info,reputest::oauth=debug cargo run
```

---

## Twitter API Debugging

### What Gets Logged

At `debug` level, the Twitter module logs:

```
[DEBUG] Building OAuth 2.0 header with token length: 1845
[DEBUG] OAuth token (masked): aGVsbG93...b3JsZA==
[INFO]  Sending POST request to Twitter API v2
[DEBUG] Request URL: https://api.x.com/2/tweets
[DEBUG] Response status: 201 Created
```

### Token Masking

Tokens are automatically masked in logs:
- First 8 characters shown
- Last 8 characters shown  
- Middle replaced with `...`

Example: `aGVsbG93...b3JsZA==`

### Debugging 401 Errors

1. **Enable debug logging**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Look for these log entries**:
   ```
   [INFO]  Found access token created at 2024-01-15 10:30:00
   [DEBUG] OAuth token (masked): abc12345...xyz98765
   [INFO]  Sending POST request to Twitter API v2
   [INFO]  Received response with status: 401 Unauthorized
   [DEBUG] Response body: {"errors":[{"code":89,"message":"Invalid or expired token."}]}
   ```

3. **Check token freshness**:
   ```sql
   SELECT id, created_at, NOW() - created_at as age
   FROM access_tokens 
   ORDER BY created_at DESC LIMIT 1;
   ```

### Testing API Calls Directly

```bash
# Get your access token (be careful - don't log this!)
ACCESS_TOKEN=$(psql $DATABASE_URL -t -c "SELECT token FROM access_tokens ORDER BY created_at DESC LIMIT 1")

# You'll need to decrypt it first since it's encrypted
# Use the application to make calls instead
```

---

## Database Debugging

### Connection Pool

```bash
RUST_LOG=sqlx=debug cargo run
```

### Query Logging

```bash
# Log all SQL queries
RUST_LOG=sqlx::query=debug cargo run
```

### Useful Diagnostic Queries

```sql
-- Check token ages
SELECT 
    'access' as type,
    created_at,
    NOW() - created_at as age
FROM access_tokens
UNION ALL
SELECT 
    'refresh' as type,
    created_at,
    NOW() - created_at as age
FROM refresh_tokens
ORDER BY created_at DESC;

-- Count good vibes by day
SELECT 
    DATE(created_at) as day,
    COUNT(*) as vibes
FROM good_vibes
GROUP BY DATE(created_at)
ORDER BY day DESC
LIMIT 7;

-- Top emitters (most good vibes sent)
SELECT 
    u.username,
    COUNT(*) as vibes_sent
FROM good_vibes gv
JOIN users u ON gv.emitter_id = u.id
GROUP BY u.username
ORDER BY vibes_sent DESC
LIMIT 10;

-- Check for orphaned records
SELECT COUNT(*) as orphaned_vibes
FROM good_vibes
WHERE emitter_id NOT IN (SELECT id FROM users)
   OR sensor_id NOT IN (SELECT id FROM users);
```

---

## Cronjob Debugging

### Trigger Immediately

The cronjob runs every 5 minutes. To test immediately:

```rust
// In src/cronjob.rs, temporarily change:
"0 0/5 * * * * *"  // Every 5 minutes
// To:
"0/10 * * * * * *" // Every 10 seconds
```

### Log Output

Normal operation at `info` level:
```
[INFO] Starting scheduled search for #gmgv tweets
[INFO] Found 3 tweets with #gmgv hashtag
[INFO] Storing good vibes: @alice -> @bob
[INFO] Scheduled search completed successfully
```

### Simulating Mentions

To test mention processing without waiting:

```sql
-- Insert a test vibe request (won't affect real data)
INSERT INTO vibe_requests (tweet_id) VALUES ('test_12345');

-- Check it was inserted
SELECT * FROM vibe_requests WHERE tweet_id LIKE 'test_%';

-- Clean up
DELETE FROM vibe_requests WHERE tweet_id LIKE 'test_%';
```

---

## Encryption Debugging

### Validate Key Format

```bash
# Check key length (should be 64 hex chars = 32 bytes)
echo -n "$TOKEN_ENCRYPTION_KEY" | wc -c
# Expected: 64

# Verify it's valid hex
echo "$TOKEN_ENCRYPTION_KEY" | grep -E '^[0-9a-fA-F]{64}$'
```

### Test Encryption Roundtrip

```bash
# Use the encrypt_token binary
echo "test_token_value" | cargo run --bin encrypt_token
```

### Common Encryption Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `not valid hex` | Key contains non-hex chars | Regenerate with `openssl rand -hex 32` |
| `must be exactly 32 bytes` | Key wrong length | Check for extra spaces/newlines |
| `Decryption failed` | Key changed | Re-authorize with current key |

---

## Performance Profiling

### Basic Timing

```bash
# Time a request
time curl http://localhost:3000/health

# Time database query
time psql $DATABASE_URL -c "SELECT COUNT(*) FROM good_vibes"
```

### Request Tracing

The app includes `TraceLayer` middleware. At `debug` level:

```
[DEBUG] request{method=GET path=/health}: started
[DEBUG] request{method=GET path=/health}: completed status=200 latency=1.2ms
```

---

## Fly.io Debugging

### SSH into Machine

```bash
fly ssh console
```

### Check Environment

```bash
fly ssh console -C "env | grep -E '(DATABASE|TOKEN|PORT|RUST)'"
```

### Live Log Streaming

```bash
# All logs
fly logs

# Filter errors
fly logs | grep -E '(ERROR|WARN)'

# Follow specific instance
fly logs --instance INSTANCE_ID
```

### Force Restart

```bash
fly machine restart
```

---

## Debug Checklist

When something isn't working:

1. [ ] Check logs at `debug` level
2. [ ] Verify environment variables are set
3. [ ] Test database connection
4. [ ] Check token freshness
5. [ ] Verify Twitter API status
6. [ ] Review recent code changes
7. [ ] Check Fly.io machine status (if deployed)

