# Klask Project Development Makefile

.PHONY: help install-hooks
.PHONY: rust-dev rust-dev-db rust-dev-full rust-test rust-clean rust-build rust-docker-up rust-docker-down rust-migrate rust-setup

# Default target
help:
	@echo "Klask Project Development Commands:"
	@echo ""
	@echo "Git Hooks:"
	@echo "  install-hooks    - Install git hooks"
	@echo ""
	@echo "Rust Backend (klask-rs):"
	@echo "  rust-dev         - Start Rust development server (requires running database)"
	@echo "  rust-dev-db      - Start only PostgreSQL database"
	@echo "  rust-dev-full    - Start database + tools (pgAdmin, Redis)"
	@echo "  rust-test        - Run all Rust tests"
	@echo "  rust-migrate     - Run database migrations by hand (the backend also does it at startup)"
	@echo "  rust-build       - Build the Rust application"
	@echo "  rust-clean       - Clean Rust build artifacts"
	@echo "  rust-docker-up   - Start all Docker services"
	@echo "  rust-docker-down - Stop all Docker services"
	@echo "  rust-setup       - Complete setup for new developers"
	@echo ""

# Git hooks
# Automatically run pre-commit and pre-push checks to catch issues early
install-hooks:
	@echo "📦 Installing Git hooks from git-hooks/..."
	git config core.hooksPath git-hooks
	chmod +x git-hooks/*
	@echo ""
	@echo "✅ Git hooks installed!"
	@echo "   • pre-commit: Runs cargo fmt, clippy, tests before each commit"
	@echo "   • pre-push: Runs cargo clippy -- -D warnings before each push"
	@echo ""
	@echo "💡 To bypass hooks if needed:"
	@echo "   git commit --no-verify"
	@echo "   git push --no-verify"

# Rust development commands
rust-dev:
	cd klask-rs && cargo run

rust-dev-db:
	docker compose -f docker-compose.dev.yml up -d postgres

rust-dev-full:
	docker compose -f docker-compose.dev.yml --profile tools up -d postgres pgadmin redis

# Rust testing
rust-test:
	cd klask-rs && cargo test

# Rust database operations
# Migrations are applied automatically when the backend starts; this target is
# only needed to run them without starting the server (requires sqlx-cli and a
# DATABASE_URL in klask-rs/.env).
rust-migrate:
	@echo "Waiting for database to be ready..."
	@until docker compose -f docker-compose.dev.yml exec postgres pg_isready -U klask_user -d klask_dev; do sleep 1; done
	cd klask-rs && sqlx migrate run

# Rust build commands
rust-build:
	cd klask-rs && cargo build --release

rust-clean:
	cd klask-rs && cargo clean
	docker compose -f docker-compose.dev.yml down -v

# Rust Docker operations
rust-docker-up:
	docker compose -f docker-compose.dev.yml up -d --build

rust-docker-down:
	docker compose -f docker-compose.dev.yml down

# Rust setup for new developers
rust-setup: rust-dev-db
	@echo "Waiting for database to be ready..."
	@sleep 5
	@echo ""
	@echo "Rust setup complete! You can now run 'make rust-dev' to start the server."
	@echo "Database: postgres://klask_user:klask_password@localhost:5433/klask_dev"
