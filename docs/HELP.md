# Getting X (Twitter) API Credentials

This guide explains how to obtain the necessary credentials from the X Developer Portal to use with the Reputest service.

## Overview

The Reputest service uses OAuth 2.0 Bearer token authentication for X API v2 endpoints.

## Prerequisites

1. An X (Twitter) account
2. Access to the [X Developer Portal](https://developer.twitter.com/)

## Step 1: Create a Developer Account

1. Go to [https://developer.twitter.com/](https://developer.twitter.com/)
2. Sign in with your X account
3. If you don't have a developer account, you'll need to apply for one
4. Complete the developer application process (this may take some time for approval)

## Step 2: Create a New App

1. Once approved, go to the [Developer Portal Dashboard](https://developer.twitter.com/en/portal/dashboard)
2. Click **"Create App"** or **"New Project"**
3. Fill out the required information:
   - **App Name**: Choose a descriptive name (e.g., "Reputest Service")
   - **App Description**: Describe what your app does
   - **Website URL**: Your website or a placeholder URL
   - **Callback URL**: For OAuth (can be `http://localhost:3000` for testing)
   - **Terms of Service**: Link to your terms or a placeholder
   - **Privacy Policy**: Link to your privacy policy or a placeholder

## Step 3: Get Your API Keys and Tokens

### For Bearer Token (OAuth 2.0)

1. Navigate to (["Projects and Apps"](https://developer.x.com/en/portal/projects-and-apps))
2. Locate your App within your Project
3. Navigate to your App's **"Keys and tokens"** tab (identified by a key icon)
4. Locate the **"Bearer Token"** field under **"Authentication Tokens"**
5. Click the **"Generate"** button to create a new Bearer Token
6. Copy this token for API operations

## Step 4: Configure App Permissions

1. Go to the **"App Settings"** tab
2. Click **"Set up"** under "User authentication settings"
3. Choose **OAuth 2.0** authentication method
4. Set the appropriate permissions:
   - **Read** for reading tweets
   - **Write** for posting tweets
   - **Read and Write** for full functionality

## Step 5: Set Environment Variables

### For OAuth 2.0 Bearer Token

Add this to your environment or `.env` file:

```bash
# OAuth 2.0 Bearer token for v2 endpoints
export xapi_bearer_token="your_bearer_token_here"
```

## Important Notes

### Rate Limits

- **OAuth 2.0 Bearer Token**: 75 requests per 15-minute window for most endpoints
- **Tweet Posting**: 300 tweets per 3-hour window

### Security Best Practices

1. **Never commit credentials to version control**
2. **Use environment variables** for all sensitive data
3. **Rotate keys regularly** for security
4. **Use the least privileged permissions** necessary
5. **Monitor your API usage** in the developer portal

### Troubleshooting

#### Common Issues

1. **"Invalid credentials" error**
   - Verify the Bearer token is correct
   - Check for extra spaces or characters
   - Ensure the token hasn't been regenerated

2. **"Rate limit exceeded" error**
   - Wait for the rate limit window to reset
   - Implement exponential backoff in your code
   - Consider upgrading to a higher tier if needed

3. **"App not found" error**
   - Verify your app is approved and active
   - Check that you're using the correct app's credentials

#### Testing Your Credentials

You can test your credentials using curl:

```bash
# Test the service (requires the service to be running)
curl -X POST http://localhost:3000/tweet

# Test Bearer token directly
curl -H "Authorization: Bearer YOUR_BEARER_TOKEN" \
     "https://api.x.com/2/users/by/username/your_username"
```

## Current Implementation

The Reputest service is already configured to use OAuth 2.0 Bearer token authentication:

1. **Authentication method** in `src/twitter.rs` uses Bearer tokens
2. **HTTP headers** use `Authorization: Bearer TOKEN`
3. **Environment variable handling** in `src/config.rs` expects `xapi_bearer_token`
4. **API endpoints** use X API v2 endpoints for both read and write operations

## Additional Resources

- [X API Documentation](https://developer.twitter.com/en/docs/twitter-api)
- [OAuth 2.0 Guide](https://developer.twitter.com/en/docs/authentication/oauth-2-0)
- [Rate Limiting Information](https://developer.twitter.com/en/docs/twitter-api/rate-limits)

## Support

If you encounter issues:

1. Check the [X API Status Page](https://api.twitterstat.us/)
2. Review the [X Developer Community](https://twittercommunity.com/)
3. Consult the [API Documentation](https://developer.twitter.com/en/docs)
4. Check your app's usage and limits in the developer portal

---

**Remember**: Keep your API credentials secure and never share them publicly!
