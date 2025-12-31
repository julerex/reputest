# CloudFlare Domain Setup with Fly.io

This guide explains how to connect your CloudFlare domain to your Fly.io application (`reputest`).

## Prerequisites

- A domain registered with CloudFlare
- Fly.io CLI installed and authenticated (`make fly-login`)
- Your app deployed to Fly.io (`make fly-deploy`)

## Step 1: Add Custom Domain to Fly.io

First, add your domain to your Fly.io application:

```bash
fly domains add `yourdomain.com`
```

Replace `yourdomain.com` with your actual domain name. You can also add a subdomain:

```bash
fly domains add `www.yourdomain.com`
fly domains add `api.yourdomain.com`
```

## Step 2: Verify Domain Addition

Check that your domain was added successfully:

```bash
fly domains list
```

You should see your domain(s) listed with a status indicating they're being configured.

## Step 3: Update CloudFlare DNS Records

Fly.io will provide you with the DNS records you need to add to CloudFlare. Run this command to see the required DNS configuration:

```bash
fly domains show `yourdomain.com`
```

This will display the CNAME or A records you need to add to CloudFlare.

### DNS Record Configuration

In your CloudFlare dashboard:

1. Go to your domain's DNS settings
2. Add the DNS records provided by Fly.io:

   **For apex domain (`yourdomain.com`):**
   - Type: `A` or `AAAA`
   - Name: `@` (or leave blank)
   - Content: The IP address provided by Fly.io
   - Proxy status: **DNS only** (grey cloud icon)

   **For subdomains (`www.yourdomain.com`):**
   - Type: `CNAME`
   - Name: `www`
   - Content: The target provided by Fly.io
   - Proxy status: **DNS only** (grey cloud icon)

**Important:** Set the proxy status to **DNS only** (grey cloud icon) for all Fly.io DNS records. Fly.io handles SSL termination, so CloudFlare's proxy should be disabled for these records.

## Step 4: Wait for DNS Propagation

DNS changes can take up to 24-48 hours to propagate globally, but often work within a few minutes. You can check the status:

```bash
fly domains list
```

## Step 5: Verify SSL Certificate

Fly.io automatically provisions SSL certificates for your custom domains. Once DNS is configured, SSL should work automatically.

Test your domain:

```bash
curl -I `https://yourdomain.com`
```

You should see a `200 OK` response with proper SSL headers.

## Step 6: Update Fly.io Configuration (Optional)

If you want to configure additional domain-specific settings, you can update your `fly.toml`:

```toml
[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = 'stop'
  auto_start_machines = false
  min_machines_running = 1
  processes = ['app']

  # Add custom domains
  [[http_service.domains]]
    name = "`yourdomain.com`"
    internal = false

  [[http_service.domains]]
    name = "`www.yourdomain.com`"
    internal = false
```

Then redeploy:

```bash
make fly-deploy
```

## Troubleshooting

### DNS Not Resolving

- Check CloudFlare DNS records are correct
- Ensure proxy status is set to "DNS only" (grey cloud)
- Wait for DNS propagation (can take up to 48 hours)
- Use tools like `dig` or `nslookup` to verify DNS records

### SSL Certificate Issues

- Fly.io handles SSL automatically - no manual certificate upload needed
- Ensure your domain is verified in Fly.io (`fly domains list`)
- Check that DNS is pointing correctly to Fly.io

### Mixed Content Warnings

If your app serves HTTP resources, ensure all internal links use HTTPS since `force_https = true` is enabled in your Fly.io config.

## Makefile Commands

Your Makefile includes these Fly.io commands:

```bash
make fly-login          # Authenticate with Fly.io
make fly-deploy         # Deploy your app
make fly-status         # Check app status
make fly-domains-list   # List configured domains
```

## Additional Resources

- [Fly.io Custom Domains Documentation](https://fly.io/docs/app-guides/custom-domains/)
- [CloudFlare DNS Management](https://developers.cloudflare.com/dns/manage-dns-records/)
- [Fly.io SSL/TLS Documentation](https://fly.io/docs/security/tls/)
