# Go workspace Makefile for protosol

.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: go-lint
go-lint: ## Run golangci-lint on all Go modules
	@echo "Running golangci-lint..."
	@if ! command -v golangci-lint &> /dev/null; then \
		echo "golangci-lint not found. Installing..."; \
		go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest; \
	fi
	golangci-lint run ./...

.PHONY: go-lint-fix
go-lint-fix: ## Run golangci-lint with auto-fix
	@echo "Running golangci-lint with auto-fix..."
	@if ! command -v golangci-lint &> /dev/null; then \
		echo "golangci-lint not found. Installing..."; \
		go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest; \
	fi
	golangci-lint run --fix ./...

.PHONY: go-fmt
go-fmt: ## Format all Go code
	@echo "Formatting Go code..."
	go fmt ./...
	goimports -w -local github.com/BRBussy/protosol .

.PHONY: go-test
go-test: ## Run all Go tests
	@echo "Running Go tests..."
	go test ./... -v

.PHONY: go-mod-tidy
go-mod-tidy: ## Tidy all Go modules
	@echo "Tidying Go modules..."
	@for dir in $$(find . -name go.mod -not -path "./vendor/*" -not -path "./lib/go/protosol/*" | xargs dirname); do \
		echo "Tidying $$dir"; \
		(cd $$dir && go mod tidy); \
	done

.PHONY: go-mod-download
go-mod-download: ## Download all Go dependencies
	@echo "Downloading Go dependencies..."
	@for dir in $$(find . -name go.mod -not -path "./vendor/*" -not -path "./lib/go/protosol/*" | xargs dirname); do \
		echo "Downloading dependencies for $$dir"; \
		(cd $$dir && go mod download); \
	done

.PHONY: go-clean
go-clean: ## Clean Go build artifacts and cache
	@echo "Cleaning Go artifacts..."
	go clean -cache -testcache -modcache