.PHONY: help build test fmt lint check clean doc
.DEFAULT_GOAL := help

GREEN := \033[32m
YELLOW := \033[33m
RESET := \033[0m

help: ## Show available commands
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-10s$(RESET) %s\n", $$1, $$2}'

build: ## Build the crate
	@echo "$(YELLOW)Building...$(RESET)"
	cargo build

test: ## Run unit and integration tests
	@echo "$(YELLOW)Running tests...$(RESET)"
	cargo test --all-targets

fmt: ## Format code with rustfmt
	@echo "$(YELLOW)Formatting...$(RESET)"
	cargo fmt --all

lint: ## Run clippy (warnings treated as errors)
	@echo "$(YELLOW)Running clippy...$(RESET)"
	cargo clippy --all-targets -- -D warnings

check: ## Run the same checks as CI (fmt --check, clippy, test)
	cargo fmt --all -- --check
	$(MAKE) lint
	$(MAKE) test

doc: ## Build and open the rustdoc API docs
	cargo doc --no-deps --open

clean: ## Remove build artifacts
	cargo clean
