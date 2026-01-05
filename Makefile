# Makefile for reputest deployment

# Default configuration (override via env vars or config.env)
DOCKER_IMAGE_NAME ?= reputest
DOCKER_IMAGE_TAG ?= latest
CONTAINER_PORT ?= 3000
CONTAINER_RUST_LOG ?= info
CONTAINER_CPU ?= 0.5
CONTAINER_MEMORY ?= 0.5
FLY_DB_CLUSTER_ID ?= your-cluster-id

-include config.env

# Derived variables
FULL_IMAGE_NAME = $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG)

DOCKER = docker

# Default target
.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' Makefile | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Prerequisites
.PHONY: check-prereqs
check-prereqs: ## Check if required tools are installed
	@echo "Checking prerequisites..."
	@command -v $(DOCKER) >/dev/null 2>&1 || { echo "Docker is required but not installed. Aborting." >&2; exit 1; }
	@echo "All prerequisites are installed."

# Development workflow
.PHONY: dev-build
dev-build: docker-build ## Build for development

.PHONY: dev-deploy
dev-deploy: docker-build ## Complete development deployment (build)

.PHONY: prod-deploy
prod-deploy: check-prereqs docker-build ## Complete production deployment with checks

# Docker targets ############################################################

.PHONY: docker-install
docker-install: ## Install Docker
	@echo "Install Docker using instructions at:"
	@echo "https://docs.docker.com/engine/install/ubuntu/#install-using-the-repository"
	@echo "The repositories will be automatically updated when installing Docker"

.PHONY: docker-status
docker-status: ## Show Docker container status
	@sudo systemctl status docker || echo "Docker is not running"

.PHONY: docker-clean
docker-clean: ## Clean up local Docker images
	@ sudo $(DOCKER) rmi $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) $(FULL_IMAGE_NAME) || true

.PHONY: docker-build 
docker-build: ## Build Docker image
	sudo $(DOCKER) build -t $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) .

.PHONY: docker-info
info: ## Show deployment information
	@echo "Configuration:"
	@echo "  Image: $(FULL_IMAGE_NAME)"
	@echo "  Port: $(CONTAINER_PORT)"
	@echo "  CPU: $(CONTAINER_CPU)"
	@echo "  Memory: $(CONTAINER_MEMORY) GB"
	@echo "  Rust Log Level: $(CONTAINER_RUST_LOG)"

# Fly targets ############################################################

.PHONY: fly-launch
fly-launch: ## Launch Fly app
	fly launch

.PHONY: fly-login
fly-login: ## Login to Fly
	fly auth login

.PHONY: fly-deploy
fly-deploy: ## Deploy to Fly
	fly deploy

.PHONY: fly-status
fly-status: ## Show Fly app status
	fly status

.PHONY: fly-machines-list
fly-machines-list: ## Show Fly machines list
	fly machines list

.PHONY: fly-certs-list
fly-certs-list: ## Show Fly certs list
	fly certs list

.PHONY: fly-ips-list
fly-ips-list: ## Show Fly IPs list
	fly ips list


# Bot targets ############################################################

.PHONY: bot-auth
bot-auth: ## Run Twitter bot authorization script
	cargo run --bin authorize_bot

.PHONY: bot-refresh-access-token
bot-refresh-access-token: ## Refresh Twitter bot access token
	cargo run --bin refresh_token

.PHONY: bot-post-tweet
bot-post-tweet: ## Post a tweet using interactive script
	cargo run --bin post_tweet


# Database targets ############################################################
# NOTE: The FLY_DB_CLUSTER_ID is specific to the maintainer's Fly.io deployment.
# If you fork this repo, override it with your own cluster ID:
#   export FLY_DB_CLUSTER_ID=your-cluster-id
# Get yours with: fly postgres list

.PHONY: fly-db-connect
fly-db-connect: ## Connect to database
	fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user

.PHONY: fly-db-counts
fly-db-counts: ## Show record counts for all tables
	@echo "SELECT tablename AS table, \
		(xpath('/row/count/text()', query_to_xml('SELECT COUNT(*) FROM ' || quote_ident(tablename), false, true, '')))[1]::text::bigint AS count \
		FROM pg_tables WHERE schemaname = 'public' ORDER BY count DESC;" \
		| fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user

.PHONY: fly-db-latest-vibes
fly-db-latest-vibe-requests: ## Show latest 20 vibe_requests records
	@echo "SELECT * FROM view_good_vibes ORDER BY created_at DESC LIMIT 20;" \
		| fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user

.PHONY: fly-db-good-vibes-all 
fly-db-good-vibes-all: ## Show all good_vibes records
	@echo "SELECT * FROM good_vibes ORDER BY created_at DESC;" \
		| fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user

.PHONY: fly-db-schema
fly-db-schema: ## Show database schema
	@echo "=== TABLES ==="
	@echo "SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename;" | fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user
	@echo
	@echo "=== INDEXES ==="
	@echo "SELECT indexname FROM pg_indexes WHERE schemaname = 'public' ORDER BY indexname;" | fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user
	@echo
	@echo "=== VIEWS ==="
	@echo "SELECT viewname FROM pg_views WHERE schemaname = 'public' ORDER BY viewname;" | fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user
	@echo
	@echo "=== MATERIALIZED VIEWS ==="
	@echo "SELECT matviewname FROM pg_matviews WHERE schemaname = 'public' ORDER BY matviewname;" | fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user




# Test targets ############################################################

# 'unset ARGV0' is needed to avoid the 
# 'error: unknown proxy name: 'Cursor-2.1.26-x86_64'; valid proxy names' error
# when running cargo and tests with Cursor.

.PHONY: test
test: ## Run tests
	unset ARGV0 && cargo test --no-fail-fast 2>&1

.PHONY: format
format: ## Format code
	unset ARGV0 && cargo fmt

.PHONY: clippy
clippy: ## Run clippy
	unset ARGV0 && cargo clippy --all-targets --all-features -- -D warnings

.PHONY: checkall
checkall: test format clippy ## Run all checks
