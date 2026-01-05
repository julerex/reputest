# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of these methods:

1. **GitHub Security Advisories**: Use [GitHub's private vulnerability reporting](https://github.com/julerex/reputest/security/advisories/new)
2. **Email**: Contact the maintainer directly (see GitHub profile)

### What to Include

Please include the following information:

- Type of vulnerability (e.g., SQL injection, XSS, authentication bypass)
- Step-by-step instructions to reproduce the issue
- Proof of concept or exploit code (if possible)
- Impact assessment
- Any suggested fixes

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity, typically within 30 days for critical issues

### What to Expect

1. **Acknowledgment**: We'll confirm receipt of your report
2. **Investigation**: We'll investigate and validate the issue
3. **Fix Development**: We'll develop and test a fix
4. **Disclosure**: We'll coordinate disclosure timing with you
5. **Credit**: We'll credit you in the security advisory (unless you prefer anonymity)

## Security Best Practices for Users

### Environment Variables

- **Never commit `.env` files** to version control
- Use secrets management (e.g., Fly.io secrets, Docker secrets)
- Rotate tokens periodically

### Token Storage

- Tokens are encrypted with AES-GCM before database storage
- Set a strong `TOKEN_ENCRYPTION_KEY` (32 bytes, hex-encoded)
- Generate with: `openssl rand -hex 32`

### Database

- Use strong, unique passwords
- Enable SSL/TLS for database connections in production
- Restrict database access to application servers only

### Deployment

- Always use HTTPS in production
- Keep dependencies updated
- Monitor for security advisories in dependencies

## Known Security Considerations

### Twitter API Tokens

- Access tokens expire and are automatically refreshed
- Refresh tokens are stored encrypted in the database
- Client secrets should never be exposed in logs or responses

### Rate Limiting

- The service includes rate limiting via `tower_governor`
- Configure appropriate limits for your use case

## Security Updates

Security updates will be released as patch versions and announced via:
- GitHub Security Advisories
- Release notes in CHANGELOG.md

