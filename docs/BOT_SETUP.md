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
   - **Callback URI / Redirect URL**: `https://reputest-small-violet-3860.fly.dev/callback`
   - **Website URL**: `https://reputest-small-violet-3860.fly.dev`

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

After running the authorization script, you'll get an access token and potentially a refresh token. Set the access token as an environment variable:

```bash
# Required: For all operations (read and write) - OAuth 2.0 User Context
export xapi_access_token="your_access_token_here"
```

**Note**: Store your refresh token securely (e.g., in your deployment platform's secrets management) for manual token refresh when needed.

## Step 4: Test Your Bot

```bash
# Test posting a tweet
curl -X POST http://localhost:3000/tweet
```

## Step 5: Handle Token Refresh (Important!)

OAuth 2.0 User Context tokens expire. You need to manually refresh them when they expire:

### Manual Token Refresh
When your bot gets a 401 error, use the refresh token utility to get a new access token:

```bash
# Use the refresh token utility
cargo run --bin refresh_token
```

The script will:
1. Ask for your Client ID and Client Secret
2. Ask for your refresh token
3. Exchange the refresh token for a new access token
4. Display the new access token for you to update

### Updating Your Access Token
After getting a new access token from the refresh script:

```bash
# Update your environment variable with the new token
export xapi_access_token="your_new_access_token_here"
```

### Storing Refresh Tokens Securely
Store your refresh token securely in your deployment platform:
- **Fly.io**: Use `fly secrets set xapi_refresh_token="your_refresh_token"`
- **Docker**: Use environment variables or Docker secrets
- **Local development**: Use `.env` files (never commit to version control)

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
