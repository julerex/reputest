# Makefile for reputest deployment

# Default configuration (override via env vars or config.env)
DOCKER_IMAGE_NAME ?= reputest
DOCKER_IMAGE_TAG ?= latest
CONTAINER_PORT ?= 3000
CONTAINER_RUST_LOG ?= info
CONTAINER_CPU ?= 0.5
CONTAINER_MEMORY ?= 0.5
FLY_DB_CLUSTER_ID ?= your-cluster-id
FLY_DB_NAME ?= reputest
# Port for fly proxy when running pg_dump (avoid clashing with local postgres)
FLY_DB_PROXY_PORT ?= 15432

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

# Compare prod schema (same queries as fly-db-schema) with sql/database_ddl.sql.
# Uses one UNION query to get type+name, parses psql output, extracts DDL object names, then diffs.
.PHONY: fly-db-schema-diff
fly-db-schema-diff: ## Diff prod schema object names vs sql/database_ddl.sql (tables, indexes, views)
	@rm -f .schema_prod.txt .schema_ddl.txt .schema_prod_sorted.txt .schema_ddl_sorted.txt
	@echo "SELECT 'TABLE' AS typ, tablename AS n FROM pg_tables WHERE schemaname = 'public' UNION ALL SELECT 'INDEX', indexname FROM pg_indexes WHERE schemaname = 'public' UNION ALL SELECT 'VIEW', viewname FROM pg_views WHERE schemaname = 'public' UNION ALL SELECT 'MATVIEW', matviewname FROM pg_matviews WHERE schemaname = 'public' ORDER BY typ, n;" \
		| fly mpg connect $(FLY_DB_CLUSTER_ID) -d reputest -u fly-user 2>/dev/null \
		| tail -n +3 | grep '|' \
		| awk -F'|' '{gsub(/^[ \t]+|[ \t]+$$/,"",$$1); gsub(/^[ \t]+|[ \t]+$$/,"",$$2); if ($$1!="" && $$2!="") print $$1 " " $$2}' \
		> .schema_prod.txt
	@grep -E 'CREATE (TABLE|INDEX|VIEW) ' sql/database_ddl.sql \
		| sed -n 's/.*CREATE \(TABLE\|INDEX\|VIEW\) \+\([a-zA-Z0-9_]*\).*/\1 \2/p' | sort -u > .schema_ddl.txt
	@sort .schema_prod.txt > .schema_prod_sorted.txt && sort .schema_ddl.txt > .schema_ddl_sorted.txt
	@echo "--- In prod only (not in sql/database_ddl.sql) ---"
	@comm -23 .schema_prod_sorted.txt .schema_ddl_sorted.txt 2>/dev/null || true
	@echo ""
	@echo "--- In sql/database_ddl.sql only (not in prod) ---"
	@comm -13 .schema_prod_sorted.txt .schema_ddl_sorted.txt 2>/dev/null || true
	@rm -f .schema_prod.txt .schema_ddl.txt .schema_prod_sorted.txt .schema_ddl_sorted.txt

# Exact DDL diff: pg_dump --schema-only from prod (via fly mpg proxy) vs sql/database_ddl.sql.
# Uses same auth as fly-db-connect: credentials from `fly mpg status -j` (requires jq).
# Requires: FLY_DB_CLUSTER_ID, jq; no DATABASE_URL needed.
.PHONY: fly-db-ddl-diff
fly-db-ddl-diff: ## Exact DDL diff: prod schema (pg_dump) vs sql/database_ddl.sql
	@command -v jq >/dev/null 2>&1 || (echo "Error: jq required (e.g. apt install jq)."; exit 1)
	@rm -f .schema_prod_ddl.sql
	@( \
		fly mpg proxy $(FLY_DB_CLUSTER_ID) -p $(FLY_DB_PROXY_PORT) & \
		PID=$$!; \
		trap "kill $$PID 2>/dev/null || true" EXIT; \
		i=0; while [ $$i -lt 30 ]; do \
			nc -z 127.0.0.1 $(FLY_DB_PROXY_PORT) 2>/dev/null && break; \
			sleep 1; i=$$((i+1)); \
			[ $$i -eq 30 ] && { echo "Error: proxy did not become ready in 30s."; exit 1; }; \
		done; \
		PROXY_URL=$$(fly mpg status $(FLY_DB_CLUSTER_ID) -j | jq -r --arg host "localhost:$(FLY_DB_PROXY_PORT)" --arg db "$(FLY_DB_NAME)" '.credentials | "postgres://\(.user):\(.password)@\($$host)/\($$db)"'); \
		pg_dump "$$PROXY_URL" --schema-only --no-owner --no-privileges -f .schema_prod_ddl.sql; \
		exit_code=$$?; \
		exit $$exit_code; \
	)
	@echo "--- diff: sql/database_ddl.sql (left) vs prod (right) ---"
	@diff -u sql/database_ddl.sql .schema_prod_ddl.sql || true
	@rm -f .schema_prod_ddl.sql




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

.PHONY: upgrade-deps
upgrade-deps: ## Upgrade all cargo dependencies to latest compatible versions (requires cargo-edit: cargo install cargo-edit)
	@command -v cargo >/dev/null 2>&1 || { echo "Error: cargo is required but not installed." >&2; exit 1; }
	@cargo --list 2>/dev/null | grep -q "upgrade" || { \
		echo "Error: cargo-edit is required but not installed." >&2; \
		echo "Install it with: cargo install cargo-edit" >&2; \
		exit 1; \
	}
	unset ARGV0 && cargo upgrade

.PHONY: checkall
checkall: test format clippy ## Run all checks

.PHONY: setup-hooks
setup-hooks: ## Install git hooks from scripts/git-hooks/
	@./scripts/setup_git_hooks.sh
