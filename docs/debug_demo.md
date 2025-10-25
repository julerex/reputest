# Enhanced Logging for Twitter API 401 Debugging

## Overview

I've added comprehensive logging to help debug 401 Unauthorized errors when using the Twitter API. The logging covers:

1. **OAuth Header Construction** (`src/oauth.rs`)
2. **Configuration Loading** (`src/config.rs`) 
3. **API Request/Response Details** (`src/twitter.rs`)

## What the Logs Will Show

### 1. Configuration Loading
```
[INFO] Loading Twitter configuration from environment variables
[INFO] Found xapi_access_token environment variable with length: 123
[DEBUG] Access token (masked): 12345678...abcdefgh
```

### 2. OAuth Header Building
```
[INFO] Building OAuth 2.0 User Context header with token length: 123
[DEBUG] OAuth token (masked): 12345678...abcdefgh
[DEBUG] Generated Authorization header: Bearer 12345678...
```

### 3. API Request Details
```
[INFO] Starting tweet post operation for text: 'Hello world'
[INFO] Target URL: https://api.x.com/2/tweets
[DEBUG] Tweet payload: {
  "text": "Hello world"
}
[INFO] Sending POST request to Twitter API v2
[DEBUG] Request URL: https://api.x.com/2/tweets
[DEBUG] Request headers: Authorization: Bearer [REDACTED], Content-Type: application/json
```

### 4. API Response Details
```
[INFO] Received response with status: 401 Unauthorized
[DEBUG] Response headers: {"content-type": "application/json", "x-rate-limit-remaining": "0"}
[ERROR] Failed to post tweet - Status: 401 Unauthorized, Response: {"errors":[{"code":89,"message":"Invalid or expired token."}]}
```

## How to Use the Logging

### 1. Set Log Level
```bash
# For detailed debugging
RUST_LOG=debug cargo run

# For normal operation
RUST_LOG=info cargo run
```

### 2. Test the Endpoints
```bash
# Test tweet posting
curl -X POST http://localhost:3000/tweet

# Test search (via cronjob)
# The search function is called automatically by the cronjob
```

### 3. Common 401 Error Scenarios

The enhanced logging will help identify:

- **Missing Token**: "Missing xapi_access_token environment variable"
- **Empty Token**: "Access token is empty"
- **Invalid Token**: "Invalid or expired token" in API response
- **Wrong Token Format**: Token length warnings
- **Expired Token**: API returns 401 with specific error message

## Security Notes

- Tokens are masked in logs (only first 8 and last 8 characters shown)
- Full tokens are never logged
- Debug logs should not be enabled in production

## Next Steps

1. Set your `xapi_access_token` environment variable
2. Run with `RUST_LOG=debug` to see detailed logs
3. Check the logs for specific error messages when 401 occurs
4. Verify token validity and permissions in Twitter Developer Portal
