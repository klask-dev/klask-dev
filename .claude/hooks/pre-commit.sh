#!/bin/bash
# Pre-commit hook for Klask
# Automatically format, lint, and test code before commits

set -e  # Exit on error

echo "🔍 Running pre-commit checks for Klask..."

# Detect which part of the project changed
CHANGED_RUST=false
CHANGED_REACT=false

if git diff --cached --name-only | grep -q "^klask-rs/"; then
    CHANGED_RUST=true
fi

if git diff --cached --name-only | grep -q "^klask-react/"; then
    CHANGED_REACT=true
fi

# Rust backend checks
if [ "$CHANGED_RUST" = true ]; then
    echo ""
    echo "🦀 Checking Rust backend..."

    cd klask-rs

    # Format code
    echo "  ├─ Formatting code..."
    cargo fmt --check || {
        echo "  │  ⚠️  Code not formatted. Running cargo fmt..."
        cargo fmt
        echo "  │  ✅ Code formatted"
    }

    # Run clippy
    echo "  ├─ Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings || {
        echo "  │  ❌ Clippy found issues. Please fix them before committing."
        cd ..
        exit 1
    }

    # Run tests
    echo "  ├─ Running tests..."
    cargo test --quiet || {
        echo "  │  ❌ Tests failed. Please fix them before committing."
        cd ..
        exit 1
    }

    echo "  └─ ✅ Rust checks passed"
    cd ..
fi

# React frontend checks
if [ "$CHANGED_REACT" = true ]; then
    echo ""
    echo "⚛️  Checking React frontend..."

    cd klask-react

    # Lint and fix
    echo "  ├─ Running ESLint..."
    npm run lint:fix || {
        echo "  │  ❌ ESLint found issues that couldn't be auto-fixed."
        cd ..
        exit 1
    }

    # Run tests
    echo "  ├─ Running tests..."
    npm test -- --run --reporter=basic || {
        echo "  │  ❌ Tests failed. Please fix them before committing."
        cd ..
        exit 1
    }

    # Type check
    echo "  ├─ Type checking..."
    npm run type-check 2>/dev/null || npx tsc --noEmit || {
        echo "  │  ❌ TypeScript errors found."
        cd ..
        exit 1
    }

    echo "  └─ ✅ React checks passed"
    cd ..
fi

echo ""
echo "✅ All pre-commit checks passed! Ready to commit."
