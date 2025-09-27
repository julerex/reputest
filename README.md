# Reputest - Rust Azure Function

A simple Rust application that runs on Azure Functions and prints "Reputesting!" to the logs. This project includes GitHub Actions for CI/CD and can be deployed to Azure Functions.

## Features

- 🦀 Rust-based Azure Function
- 📝 Logs "Reputesting!" message
- 🚀 GitHub Actions CI/CD pipeline
- ☁️ Azure Functions deployment ready
- 🔧 Local development support

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Azure Functions Core Tools](https://docs.microsoft.com/en-us/azure/azure-functions/functions-run-local)
- [Azure CLI](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli) (for deployment)
- [Node.js](https://nodejs.org/) (for Azure Functions Core Tools)

## Local Development

### 1. Clone and Setup

```bash
git clone <your-repo-url>
cd reputest
```

### 2. Install Dependencies

#### Install/Update Azure Functions Core Tools
(on Ubuntu)


```bash
# add the Microsoft package repository for your Ubuntu version
wget -q https://packages.microsoft.com/config/ubuntu/22.04/packages-microsoft-prod.deb
sudo dpkg -i packages-microsoft-prod.deb

# Update the package list to refresh the Microsoft repository feed
sudo apt-get update

# sudo apt-get install azure-functions-core-tools-4
sudo apt-get upgrade azure-functions-core-tools-4

# Verify the upgrade
func --version
```

#### Build the project

```bash
# Build the project
cargo build --release
```

### 3. Run Locally

```bash
# Start the Azure Functions runtime
func start
```

The function will be available at `http://localhost:7071/api/reputest`

### 4. Test the Function

```bash
# Test with curl
curl http://localhost:7071/api/reputest

# Or visit in your browser
open http://localhost:7071/api/reputest
```

## Deployment to Azure

### 1. Create Azure Resources

```bash
# Login to Azure
az login

# Create a resource group
az group create --name myResourceGroup --location eastus

# Create a storage account
az storage account create \
  --name mystorageaccount \
  --location eastus \
  --resource-group myResourceGroup \
  --sku Standard_LRS

# Create the function app
az functionapp create \
  --resource-group myResourceGroup \
  --consumption-plan-location eastus \
  --runtime custom \
  --runtime-version 3.1 \
  --functions-version 4 \
  --name myFunctionApp \
  --storage-account mystorageaccount
```

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository:

- `AZURE_CREDENTIALS`: Azure service principal credentials
- `AZURE_FUNCTIONAPP_PUBLISH_PROFILE`: Function app publish profile

To get the credentials:

```bash
# Create service principal
az ad sp create-for-rbac --name "myApp" --role contributor \
  --scopes /subscriptions/{subscription-id}/resourceGroups/{resource-group} \
  --sdk-auth

# Get publish profile
az functionapp deployment list-publishing-profiles \
  --name myFunctionApp \
  --resource-group myResourceGroup \
  --xml
```

### 3. Update Configuration

1. Update `AZURE_FUNCTIONAPP_NAME` in `.github/workflows/ci-cd.yml`
2. Update `AZURE_FUNCTIONAPP_NAME` in `azure-pipelines.yml` (if using Azure DevOps)

### 4. Deploy

Push to the `main` branch to trigger automatic deployment:

```bash
git add .
git commit -m "Initial commit"
git push origin main
```

## Project Structure

```
reputest/
├── src/
│   └── main.rs              # Main Rust function
├── azure-functions/
│   └── function.json        # Azure Functions configuration
├── .github/
│   └── workflows/
│       └── ci-cd.yml        # GitHub Actions workflow
├── Cargo.toml               # Rust dependencies
├── host.json                # Azure Functions host configuration
├── local.settings.json      # Local development settings
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
- 🚀 Automatic deployment to Azure Functions

## Customization

### Changing the Message

Edit `src/main.rs` to change the logged message:

```rust
info!("Your custom message here!");
```

### Adding More Functions

1. Add new functions to `src/main.rs`
2. Create corresponding `function.json` files in the `azure-functions/` directory
3. Update the Azure Functions configuration as needed

## Troubleshooting

### Common Issues

1. **Build fails**: Ensure you have the latest Rust toolchain
2. **Deployment fails**: Check Azure credentials and function app name
3. **Function not responding**: Verify the function.json configuration

### Logs

View function logs in the Azure portal or using Azure CLI:

```bash
az functionapp logs tail --name myFunctionApp --resource-group myResourceGroup
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
