# Twitter Bot Setup Guide

This guide will help you set up OAuth 2.0 User Context authentication for your Twitter bot.

## Prerequisites

1. **Twitter Developer Account**: You need a Twitter Developer account
2. **Twitter App**: Create a Twitter App in the Developer Portal
3. **OAuth 2.0 Settings**: Configure your app for OAuth 2.0

## Step 1: Configure Your Twitter App

1. Go to the [Twitter Developer Portal](https://developer.twitter.com/)
2. Under the "Projects and Apps" dropdown, select your Project
3. Under **Apps** section, locate your Project and select "App settings" (the gear icon)
4. You should be directed to the **Settings** tab of your App
3. Under the **User authentication settings** section, select the **Edit** button.
4. Enable OAuth 2.0
5. Set the following:
   - **App Type**: Single Page App (for PKCE) / "Web App, Automated App or Bot"
   - **Callback URI / Redirect URL**: `https://reputest.fly.dev/callback`
   - **Website URL**: `https://reputest.fly.dev`

6. Save your settings and note down:
   - **Client ID**
   - **Client Secret**

## Step 2: Run the Authorization Script

The authorization script will help you get the access token for your bot.

```bash
# Build and run the authorization script
cargo run --bin authorize_bot
```

The script will:
1. Ask for your Client ID and Client Secret
2. Ask for your Redirect URI
3. Generate a secure authorization URL
4. Guide you through the browser authorization process
5. Exchange the authorization code for an access token

## Step 3: Set Environment Variables

After running the authorization script, you'll get an access token and potentially a refresh token. Set them as environment variables:

```bash
# Required: For all operations (read and write) - OAuth 2.0 User Context
export xapi_access_token="your_access_token_here"

# Optional: For automatic token refresh (recommended)
export xapi_refresh_token="your_refresh_token_here"
export XAPI_CLIENT_ID="your_client_id_here"
export XAPI_CLIENT_SECRET="your_client_secret_here"
```

**Note**: If you set the refresh token and client credentials, your bot will automatically refresh expired access tokens without manual intervention.

## Step 4: Test Your Bot

```bash
# Test posting a tweet
curl -X POST http://localhost:3000/tweet
```

## Step 5: Token Refresh (Automatic!)

OAuth 2.0 User Context tokens expire. Your bot now handles this automatically:

### Automatic Token Refresh (Default Behavior)
If you've set the required environment variables (`xapi_refresh_token`, `XAPI_CLIENT_ID`, `XAPI_CLIENT_SECRET`), your bot will:

1. **Detect 401 errors** automatically when making API calls
2. **Refresh the access token** using the stored refresh token
3. **Retry the failed request** with the new token
4. **Log the entire process** for debugging

### Manual Refresh (Fallback)
If automatic refresh is not configured or fails, you can manually refresh tokens:

```bash
# Use the refresh token utility
cargo run --bin refresh_token
```

### Monitoring Token Refresh
The bot logs all token refresh activities:
- When tokens are refreshed automatically
- When refresh fails and manual intervention is needed
- Token expiration times and refresh success/failure

## Alternative: OAuth 1.0a for Bots

If you prefer a simpler approach for bots, you can also use OAuth 1.0a:

1. In your Twitter App settings, enable OAuth 1.0a
2. Generate Access Token and Access Token Secret
3. Use these credentials directly (no authorization flow needed)

However, OAuth 2.0 User Context is the recommended approach for new applications.

## Security Best Practices

1. **Never commit tokens to version control**
2. **Use environment variables or secure secret management**
3. **Rotate tokens regularly**
4. **Monitor your app's usage in the Twitter Developer Portal**
5. **Implement proper error handling and rate limiting**

## Troubleshooting

### Common Issues:

1. **"Invalid client" error**: Check your Client ID and Client Secret
2. **"Invalid redirect URI"**: Ensure the redirect URI matches exactly
3. **"Invalid scope"**: Make sure you've selected the correct scopes in your app settings
4. **"Token expired"**: Re-run the authorization script to get a new token

### Getting Help:

- Check the [Twitter API Documentation](https://developer.twitter.com/en/docs)
- Review your app settings in the Developer Portal
- Check the error messages in your bot's logs

## Next Steps

Once your bot is authenticated:

1. Implement your bot's logic
2. Add proper error handling
3. Set up monitoring and logging
4. Consider implementing automatic token refresh
5. Deploy your bot to a server

Your bot should now be able to post tweets using OAuth 2.0 User Context authentication!
