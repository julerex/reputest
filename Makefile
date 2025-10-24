# Makefile for reputest deployment

# Load configuration from config.env if it exists
-include magiconfig.env

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

# Utility targets ############################################################

.PHONY: info
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

.PHONY: bot-auth
bot-auth: ## Run Twitter bot authorization script
	cargo run --bin authorize_bot

.PHONY: bot-refresh
bot-refresh: ## Refresh Twitter bot access token
	cargo run --bin refresh_token