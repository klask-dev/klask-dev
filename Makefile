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
	@echo "  rust-migrate     - Run database migrations"
	@echo "  rust-build       - Build the Rust application"
	@echo "  rust-clean       - Clean Rust build artifacts"
	@echo "  rust-docker-up   - Start all Docker services"
	@echo "  rust-docker-down - Stop all Docker services"
	@echo "  rust-setup       - Complete setup for new developers"
	@echo ""

# Git hooks
install-hooks:
	git config core.hooksPath git-hooks
	chmod +x git-hooks/*

# Rust development commands
rust-dev:
	cd klask-rs && cargo run

rust-dev-db:
	cd klask-rs && docker-compose up -d postgres

rust-dev-full:
	cd klask-rs && docker-compose --profile tools up -d

# Rust testing
rust-test:
	cd klask-rs && cargo test

# Rust database operations
rust-migrate:
	@echo "Waiting for database to be ready..."
	@until cd klask-rs && docker-compose exec postgres pg_isready -U klask -d klask_rs; do sleep 1; done
	cd klask-rs && sqlx migrate run

# Rust build commands
rust-build:
	cd klask-rs && cargo build --release

rust-clean:
	cd klask-rs && cargo clean
	cd klask-rs && docker-compose down -v

# Rust Docker operations
rust-docker-up:
	cd klask-rs && docker-compose up -d

rust-docker-down:
	cd klask-rs && docker-compose down

# Rust setup for new developers
rust-setup: rust-dev-db
	@echo "Waiting for database to be ready..."
	@sleep 5
	$(MAKE) rust-migrate
	@echo ""
	@echo "Rust setup complete! You can now run 'make rust-dev' to start the server."
	@echo "Database: postgres://klask:klask@localhost:5432/klask_rs"
	@echo "pgAdmin: http://localhost:8080 (admin@klask.dev / admin)"
