# Makefile for reputest Azure Container Instance deployment

# Load configuration from config.env if it exists
-include magiconfig.env

# Derived variables
FULL_IMAGE_NAME = $(AZURE_REGISTRY_NAME).azurecr.io/$(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG)

AZ = az
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
	@command -v $(AZ) >/dev/null 2>&1 || { echo "Azure CLI is required but not installed. Aborting." >&2; exit 1; }
	@command -v $(DOCKER) >/dev/null 2>&1 || { echo "Docker is required but not installed. Aborting." >&2; exit 1; }
	@echo "All prerequisites are installed."


# Azure CLI commands

# Azure setup

# Note that you need to log in AND visit the Azure portal
# Otherwise you will still get the "No subscriptions found for ..." error
.PHONY: azure-login
azure-login: ## Login to Azure
	$(AZ) login

.PHONY: azure-list-resource-groups
azure-list-resource-groups: ## List all Azure resource groups
	@echo "Azure resource groups in your subscription:"
	$(AZ) group list --output table


.PHONY: azure-list-registries
azure-list-registries: ## List all Azure Container Registries in subscription
	@echo "Azure Container Registries in your subscription:"
	$(AZ) acr list --output table


.PHONY: azure-create-resource-group
azure-create-resource-group: ## Create Azure resource group
	$(AZ) group create --name $(AZURE_RESOURCE_GROUP) --location $(AZURE_LOCATION)

.PHONY: azure-create-container-registry
azure-create-container-registry: ## Create Azure Container Registry
	$(AZ) acr create --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_REGISTRY_NAME) --sku Basic --admin-enabled true

.PHONY: azure-setup
azure-setup: azure-login azure-create-resource-group azure-create-container-registry ## Complete Azure setup (login, create RG and ACR)

# Azure Container Instance operations
.PHONY: azure-deploy-aci
azure-deploy-aci: ## Deploy container to Azure Container Instance
	$(AZ) container create \
		--resource-group $(AZURE_RESOURCE_GROUP) \
		--name $(AZURE_CONTAINER_NAME) \
		--image $(FULL_IMAGE_NAME) \
		--registry-login-server $(AZURE_REGISTRY_NAME).azurecr.io \
		--registry-username $$($(AZ) acr credential show --name $(AZURE_REGISTRY_NAME) --query username --output tsv) \
		--registry-password $$($(AZ) acr credential show --name $(AZURE_REGISTRY_NAME) --query passwords[0].value --output tsv) \
		--dns-name-label $(AZURE_DNS_NAME_LABEL) \
		--ports $(CONTAINER_PORT) \
		--environment-variables RUST_LOG=$(CONTAINER_RUST_LOG) PORT=$(CONTAINER_PORT) \
		--cpu $(CONTAINER_CPU) \
		--memory $(CONTAINER_MEMORY) \
		--restart-policy Always

.PHONY: azure-deploy-from-template
azure-deploy-from-template: ## Deploy using ARM template (aci-deployment.json)
	$(AZ) deployment group create \
		--resource-group $(AZURE_RESOURCE_GROUP) \
		--template-file aci-deployment.json \
		--parameters image=$(FULL_IMAGE_NAME) \
		--parameters registryServer=$(AZURE_REGISTRY_NAME).azurecr.io \
		--parameters registryUsername=$$($(AZ) acr credential show --name $(AZURE_REGISTRY_NAME) --query username --output tsv) \
		--parameters registryPassword=$$($(AZ) acr credential show --name $(AZURE_REGISTRY_NAME) --query passwords[0].value --output tsv)

.PHONY: azure-logs
azure-logs: ## Show container logs
	$(AZ) container logs --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_CONTAINER_NAME)

.PHONY: azure-container-status
azure-container-status: ## Show container status
	$(AZ) container show --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_CONTAINER_NAME) --query instanceView.state

.PHONY: azure-get-url-container
azure-get-url-container: ## Get the public URL of the container
	@echo "Container URL: http://$$($(AZ) container show --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_CONTAINER_NAME) --query ipAddress.fqdn --output tsv):$(CONTAINER_PORT)"

.PHONY: delete
delete: ## Delete the container instance
	$(AZ) container delete --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_CONTAINER_NAME) --yes

.PHONY: delete-all
delete-all: ## Delete container instance and resource group
	$(AZ) container delete --resource-group $(AZURE_RESOURCE_GROUP) --name $(AZURE_CONTAINER_NAME) --yes || true
	$(AZ) group delete --name $(AZURE_RESOURCE_GROUP) --yes --no-wait

# Development workflow
.PHONY: dev-build
dev-build: docker-build ## Build for development (no push)

.PHONY: dev-deploy
dev-deploy: docker-build docker-push azure-deploy-aci ## Complete development deployment (build, push, deploy)

.PHONY: prod-deploy
prod-deploy: check-prereqs docker-build docker-push azure-deploy-aci ## Complete production deployment with checks

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


# Docker operations
.PHONY: docker-build
docker-build: ## Build Docker image
	sudo $(DOCKER) build -t $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) .

.PHONY: docker-tag
docker-tag: ## Tag image for Azure Container Registry
	$(DOCKER) tag $(DOCKER_IMAGE_NAME):$(DOCKER_IMAGE_TAG) $(FULL_IMAGE_NAME)

.PHONY: azure-login-acr
azure-login-acr: ## Login to Azure Container Registry
	$(AZ) acr login --name $(AZURE_REGISTRY_NAME)

.PHONY: docker-push
docker-push: azure-login-acr docker-tag ## Push image to Azure Container Registry
	$(DOCKER) push $(FULL_IMAGE_NAME)


# Utility targets ############################################################



.PHONY: info
info: ## Show deployment information
	@echo "Configuration:"
	@echo "  Registry: $(AZURE_REGISTRY_NAME).azurecr.io"
	@echo "  Image: $(FULL_IMAGE_NAME)"
	@echo "  Resource Group: $(AZURE_RESOURCE_GROUP)"
	@echo "  Container Name: $(AZURE_CONTAINER_NAME)"
	@echo "  Location: $(AZURE_LOCATION)"
	@echo "  DNS Label: $(AZURE_DNS_NAME_LABEL)"
	@echo "  Port: $(CONTAINER_PORT)"
	@echo "  CPU: $(CONTAINER_CPU)"
	@echo "  Memory: $(CONTAINER_MEMORY) GB"
	@echo "  Rust Log Level: $(CONTAINER_RUST_LOG)"

# Quick start target
.PHONY: quickstart
quickstart: azure-setup prod-deploy azure-get-url-container ## Complete setup and deployment in one command

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