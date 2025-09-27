# Reputest - Rust Container Service

A simple Rust web service that runs on Azure Container Instances and prints "Reputesting!" to the logs. This project uses Docker containerization with warp web server and includes GitHub Actions for CI/CD.

## Features

- 🦀 Rust-based web service containerized with Docker
- 🌐 Warp web server for HTTP handling
- 📝 Logs "Reputesting!" message
- 🚀 GitHub Actions CI/CD pipeline
- ☁️ Azure Container Instances deployment ready
- 🔧 Local development support
- 🐳 Multi-stage Docker build for optimized image size

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Docker](https://docs.docker.com/get-docker/) (for containerization)
- [Azure CLI](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli) (for deployment)
- [Azure Container Registry](https://docs.microsoft.com/en-us/azure/container-registry/) (for storing container images)

## Local Development

### 1. Clone and Setup

```bash
git clone <your-repo-url>
cd reputest
```

### 2. Build and Run Locally

#### Build the Rust project

```bash
# Build the project
cargo build --release

# Run the application locally
cargo run
```

The service will be available at `http://localhost:8080`

#### Build and run with Docker

```bash
# Build the Docker image
docker build -t reputest .

# Run the container
docker run -p 8080:8080 reputest
```

### 3. Test the Service

```bash
# Test the main endpoint
curl http://localhost:8080/reputest

# Test the health check endpoint
curl http://localhost:8080/health

# Test the root endpoint
curl http://localhost:8080/

# Or visit in your browser
open http://localhost:8080/reputest
```

## Deployment to Azure

### 1. Create Azure Resources

```bash
# Login to Azure
az login

# Create a resource group
az group create --name reputest-rg --location eastus

# Create an Azure Container Registry
az acr create \
  --resource-group reputest-rg \
  --name yourregistry \
  --sku Basic \
  --admin-enabled true

# Get the ACR login server and credentials
az acr show --name yourregistry --query loginServer --output tsv
az acr credential show --name yourregistry --query passwords[0].value --output tsv
```

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository:

- `AZURE_CREDENTIALS`: Azure service principal credentials
- `ACR_USERNAME`: Azure Container Registry username (usually the registry name)
- `ACR_PASSWORD`: Azure Container Registry password

To get the credentials:

```bash
# Create service principal
az ad sp create-for-rbac --name "reputest-app" --role contributor \
  --scopes /subscriptions/{subscription-id}/resourceGroups/reputest-rg \
  --sdk-auth

# Get ACR credentials
az acr credential show --name yourregistry --query username --output tsv
az acr credential show --name yourregistry --query passwords[0].value --output tsv
```

### 3. Update Configuration

1. Update `AZURE_CONTAINER_REGISTRY` in `.github/workflows/ci-cd.yml` with your ACR login server
2. Update `RESOURCE_GROUP` and `CONTAINER_GROUP_NAME` in the workflow if needed

### 4. Deploy

Push to the `main` branch to trigger automatic deployment:

```bash
git add .
git commit -m "Initial commit"
git push origin main
```

After deployment, your service will be available at:
`http://reputest-aci.eastus.azurecontainer.io:8080/reputest`

## Project Structure

```
reputest/
├── src/
│   └── main.rs              # Main Rust web service with warp server
├── .github/
│   └── workflows/
│       └── ci-cd.yml        # GitHub Actions workflow for ACI deployment
├── Cargo.toml               # Rust dependencies (warp, tokio)
├── Dockerfile               # Multi-stage Docker build configuration
├── aci-deployment.json      # Azure Container Instance deployment template
├── azure-pipelines.yml      # Azure DevOps pipeline (alternative)
└── README.md               # This file
```

## CI/CD Pipeline

The GitHub Actions workflow includes:

- ✅ Rust toolchain setup
- 📦 Cargo caching for faster builds
- 🧪 Running tests
- 🔍 Clippy linting
- 📝 Rustfmt formatting check
- 🐳 Docker image building and pushing to Azure Container Registry
- 🚀 Automatic deployment to Azure Container Instances

## Customization

### Changing the Message

Edit `src/main.rs` to change the logged message:

```rust
info!("Your custom message here!");
```

### Adding More Endpoints

1. Add new routes to the warp server in `src/main.rs`
2. Update the Dockerfile if additional dependencies are needed
3. Test locally with `cargo run` or `docker run`

### How Container Deployment Works

This project uses Azure Container Instances, which provides a simple way to run containers in Azure without managing virtual machines. Key points:

- The `Dockerfile` creates a multi-stage build for optimized image size
- The container runs as a non-root user for security
- Environment variables control logging and port configuration
- Azure Container Registry stores the built images
- GitHub Actions automatically builds and deploys on code changes

## Troubleshooting

### Common Issues

1. **Build fails**: Ensure you have the latest Rust toolchain and Docker installed
2. **Deployment fails**: Check Azure credentials and container registry configuration
3. **Container not responding**: Verify the container is running and port 8080 is accessible

### Logs

View container logs in the Azure portal or using Azure CLI:

```bash
az container logs --name reputest-container --resource-group reputest-rg
```

### Manual Deployment

You can also deploy manually using the Azure CLI:

```bash
# Build and push the image manually
docker build -t yourregistry.azurecr.io/reputest:latest .
docker push yourregistry.azurecr.io/reputest:latest

# Deploy to Azure Container Instances
az container create \
  --resource-group reputest-rg \
  --name reputest-container \
  --image yourregistry.azurecr.io/reputest:latest \
  --registry-login-server yourregistry.azurecr.io \
  --registry-username yourregistry \
  --registry-password yourpassword \
  --dns-name-label reputest-aci \
  --ports 8080 \
  --environment-variables RUST_LOG=info PORT=8080
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
