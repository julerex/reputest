# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial open source release
- OAuth 2.0 User Context authentication for Twitter/X API
- Automatic token refresh when tokens expire (401 detection)
- Encrypted token storage in PostgreSQL database
- Hashtag monitoring cronjob (#gmgv)
- Vibe score calculation with multi-degree path analysis
- Health check endpoint
- Docker multi-stage builds
- Fly.io deployment configuration
- GitHub Actions CI/CD workflows
- Comprehensive test suite

### Security
- AES-GCM encryption for stored tokens
- PKCE flow for OAuth 2.0 authorization
- No hardcoded secrets in codebase

## [0.1.0] - 2025-01-05

### Added
- Initial release
- Basic web service with Axum framework
- Twitter/X API integration
- PostgreSQL database support
- Docker containerization

[Unreleased]: https://github.com/julerex/reputest/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/julerex/reputest/releases/tag/v0.1.0

