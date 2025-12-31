# CloudFlare Domain Setup with Fly.io

This guide explains how to connect your CloudFlare domain to your Fly.io application (`reputest`).

## Prerequisites

- A domain registered with CloudFlare
- Fly.io CLI installed and authenticated (`make fly-login`)
- Your app deployed to Fly.io (`make fly-deploy`)

## Step 1: Add Custom Domain to Fly.io

First, add your domain to your Fly.io application:

```bash
fly certs add yourdomain.com
```

Replace `yourdomain.com` with your actual domain name. You can also add a subdomain:

```bash
fly certs add www.yourdomain.com
fly certs add api.yourdomain.com
```

## Step 2: Verify Domain Addition

Check that your domain was added successfully:

```bash
fly certs list
```

You should see your domain(s) listed with a status indicating they're being configured.

## Step 3: Configure DNS Records Manually

**Important:** Fly.io's automatic CloudFlare integration only works for **subdomains** (like `www.yourdomain.com`), not for **apex domains** (like `yourdomain.com`). You need to manually configure DNS records in CloudFlare for apex domains.

### For Apex Domain (`yourdomain.com`)

In your CloudFlare dashboard:

1. Go to **DNS** → **Records**
2. Add an **A record**:
   - **Type**: `A`
   - **Name**: `@` (leave blank for apex domain)
   - **Content**: The IP address from `fly ips list` (run this command to get your app's IP)
   - **Proxy status**: **DNS only** (grey cloud icon)

```bash
# Get your Fly.io app's IP address
fly ips list
```

This will show your app's IPv4 and IPv6 addresses. Use the IPv4 address for the A record.

### For Subdomains

Subdomains work automatically with `fly certs add` - no manual DNS configuration needed.

### Verify Configuration

After setting up DNS records, verify everything is working:

```bash
fly certs check yourdomain.com
```

When successful, you'll see:

- Status: Ready
- DNS Provider: cloudflare
- Certificate Authority: Let's Encrypt
- A confirmation that "Your DNS is correctly configured"

## Step 4: Wait for DNS Propagation and SSL Setup

DNS changes and SSL certificate provisioning can take up to 24-48 hours to propagate globally, but often work within a few minutes. You can check the status:

```bash
fly certs list
fly certs check yourdomain.com
```

Fly.io automatically provisions SSL certificates for your custom domains. Once DNS is configured, SSL should work automatically. Test your domain:

```bash
curl -I https://yourdomain.com
```

You should see a `200 OK` response with proper SSL headers.

## Troubleshooting

### Apex Domain Not Working

**Problem**: `yourdomain.com` doesn't work, but `www.yourdomain.com` does.

**Solution**: Fly.io's automatic CloudFlare integration only works for subdomains. For apex domains, you must manually add DNS records in CloudFlare pointing to your Fly.io app's IP address.

### DNS Not Propagating

**Problem**: Certificate shows "Ready" but domain doesn't work.

**Solution**:

1. Wait 24-48 hours for DNS propagation
2. Verify DNS records with `dig yourdomain.com`
3. Check CloudFlare proxy status is set to "DNS only"

### Certificate Issues

**Problem**: SSL certificate errors.

**Solution**: Run `fly certs check yourdomain.com` and ensure DNS is properly configured.

## Step 5: Update Fly.io Configuration (Optional)

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

### Additional Issues

#### DNS Not Resolving

- Check CloudFlare DNS records are correct
- Ensure proxy status is set to "DNS only" (grey cloud)
- Wait for DNS propagation (can take up to 48 hours)
- Use tools like `dig` or `nslookup` to verify DNS records

### SSL Certificate Issues

- Fly.io handles SSL automatically - no manual certificate upload needed
- Ensure your domain is verified in Fly.io (`fly certs list`)
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
