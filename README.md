# Reputest - Rust Web Service with Twitter/X API Integration

A modern Rust web service built with Axum that provides HTTP endpoints for testing and demonstration purposes, featuring Twitter/X API integration using OAuth 1.0a authentication and automated hashtag monitoring via cronjobs.

## Features

- 🦀 **Rust-based web service** with Axum framework for high performance
- 🐦 **Twitter/X API integration** with OAuth 1.0a authentication
- 📅 **Automated cronjob scheduling** for hashtag monitoring (GMGV hashtag every 10 minutes)
- 🌐 **Multiple HTTP endpoints** for testing and health monitoring
- 📝 **Structured logging** with configurable log levels
- 🐳 **Docker containerization** with multi-stage builds for optimized images
- ☁️ **Multi-platform deployment** support (Fly.io)
- 🔧 **Comprehensive test suite** with async testing utilities
- 🚀 **Production-ready** with optimized release builds

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Docker](https://docs.docker.com/get-docker/) (for containerization)
- Twitter/X API credentials (for Twitter functionality)
- [Fly CLI](https://fly.io/docs/hands-on/install-flyctl/) (for Fly.io deployment)

## Environment Variables

The following environment variables are required for full functionality:

### Required for Twitter/X API Integration

- `xapi_consumer_key`: Twitter API consumer key
- `xapi_consumer_secret`: Twitter API consumer secret  
- `xapi_access_token`: Twitter API access token
- `xapi_access_token_secret`: Twitter API access token secret

### Optional Configuration

- `PORT`: Server port (defaults to 3000)
- `RUST_LOG`: Log level (defaults to info)

## API Endpoints

| Method | Endpoint | Description | Response |
|--------|----------|-------------|----------|
| `GET` | `/` | Welcome message | `"A new reputest is in the house!"` |
| `GET` | `/reputest` | Test endpoint | `"Reputesting!"` |
| `POST` | `/reputest` | Test endpoint | `"Reputesting!"` |
| `GET` | `/health` | Health check | `{"status": "healthy", "service": "reputest"}` |
| `POST` | `/tweet` | Post tweet to Twitter/X | Tweet response or error |

### Example API Usage

```bash
# Test the service
curl http://localhost:3000/reputest

# Check health
curl http://localhost:3000/health

# Post a tweet (requires Twitter API credentials)
curl -X POST http://localhost:3000/tweet
```

## Local Development

### 1. Clone and Setup

```bash
git clone <your-repo-url>
cd reputest
```

### 2. Configure Environment Variables

Create a `.env` file or set environment variables:

```bash
# Required for Twitter functionality
export xapi_consumer_key="your_consumer_key"
export xapi_consumer_secret="your_consumer_secret"
export xapi_access_token="your_access_token"
export xapi_access_token_secret="your_access_token_secret"

# Optional
export PORT=3000
export RUST_LOG=info
```

### 3. Build and Run

#### Using Cargo (Development)

```bash
# Build the project
cargo build

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run with specific port
PORT=8080 cargo run
```

#### Using Docker

```bash
# Build the Docker image
docker build -t reputest .

# Run the container
docker run -p 3000:3000 \
  -e xapi_consumer_key="your_key" \
  -e xapi_consumer_secret="your_secret" \
  -e xapi_access_token="your_token" \
  -e xapi_access_token_secret="your_token_secret" \
  reputest
```

## Deployment

### Fly.io Deployment

The project includes Fly.io configuration for easy deployment:

```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Login to Fly.io
fly auth login

# Deploy (first time)
fly launch

# Deploy updates
fly deploy

# Set environment variables
fly secrets set xapi_consumer_key="your_key"
fly secrets set xapi_consumer_secret="your_secret"
fly secrets set xapi_access_token="your_token"
fly secrets set xapi_access_token_secret="your_token_secret"
```

## Project Structure

```text
reputest/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── config.rs            # Configuration and environment handling
│   ├── handlers.rs          # HTTP route handlers
│   ├── twitter.rs           # Twitter/X API integration
│   ├── oauth.rs             # OAuth 1.0a authentication implementation
│   ├── cronjob.rs           # Scheduled task management
│   └── tests.rs             # Comprehensive test suite
├── Cargo.toml               # Rust dependencies and project metadata
├── Dockerfile               # Multi-stage Docker build configuration
├── fly.toml                 # Fly.io deployment configuration
├── Makefile                 # Build and deployment automation
├── magiconfig.env           # Environment configuration template
└── README.md               # This file
```

## Key Components

### Twitter/X API Integration

The service includes full OAuth 1.0a implementation for Twitter/X API:

- **Authentication**: Complete OAuth 1.0a flow with HMAC-SHA1 signatures
- **Tweet Posting**: Post tweets via the `/tweet` endpoint
- **Hashtag Monitoring**: Automated search for tweets with specific hashtags
- **Rate Limiting**: Proper handling of API rate limits and errors

### Cronjob System

Automated hashtag monitoring runs every 10 minutes:

- **GMGV Hashtag**: Searches for tweets containing #gmgv from the past hour
- **Logging**: All found tweets are logged with timestamps and IDs
- **Error Handling**: Graceful handling of API errors and network issues
- **Configurable**: Easy to modify schedule and hashtag via code

### HTTP Server

Built with Axum for high performance:

- **Async/Await**: Full async support for concurrent request handling
- **Middleware**: Request tracing and logging middleware
- **Error Handling**: Comprehensive error responses with proper HTTP status codes
- **Health Checks**: Built-in health monitoring endpoint

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test handlers
```

### Code Quality

The project includes comprehensive documentation and follows Rust best practices:

- **Documentation**: All public functions include detailed rustdoc comments
- **Error Handling**: Proper error propagation with custom error types
- **Testing**: Unit tests for all major functionality
- **Performance**: Optimized release builds with size optimization

### Adding New Features

1. **New Endpoints**: Add handlers in `src/handlers.rs` and register routes in `main.rs`
2. **Twitter Integration**: Extend `src/twitter.rs` with new API endpoints
3. **Scheduled Tasks**: Add new cronjobs in `src/cronjob.rs`
4. **Configuration**: Add new environment variables in `src/config.rs`

## Troubleshooting

### Common Issues

1. **Twitter API Errors**: Verify all four environment variables are set correctly
2. **Port Conflicts**: Change the `PORT` environment variable if 3000 is in use
3. **Docker Build Fails**: Ensure Docker is running and you have sufficient disk space
4. **Deployment Issues**: Check cloud provider credentials and resource limits

### Logs

View application logs:

```bash
# Local development
RUST_LOG=debug cargo run

# Docker
docker logs <container_id>

# Fly.io
fly logs

```

### Performance

The service is optimized for production:

- **Release Builds**: Uses `opt-level = "z"` for smallest binary size
- **Link Time Optimization**: Enabled for better performance
- **Single Codegen Unit**: Optimized compilation
- **Panic Abort**: Smaller binary size in production

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with proper documentation
4. Run tests (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Axum](https://github.com/tokio-rs/axum) web framework
- Twitter/X API integration using OAuth 1.0a
- Docker multi-stage builds for optimized containers
- Fly.io for deployment
