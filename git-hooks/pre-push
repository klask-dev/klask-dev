#!/bin/bash
# Pre-push hook for Klask
# Runs cargo clippy with -D warnings to prevent pushing code that fails in CI

set -e

echo "🔍 Running pre-push checks..."

# Check if we're pushing changes to Rust code
if git diff --name-only remotes/origin/master HEAD 2>/dev/null | grep -q "^klask-rs/" || \
   git diff --name-only HEAD~1 HEAD 2>/dev/null | grep -q "^klask-rs/"; then

    echo ""
    echo "🦀 Running cargo clippy with strict warnings..."
    echo "   (This ensures we don't push code that breaks CI)"
    echo ""

    cd klask-rs

    # Run clippy with -D warnings - this is what GitHub Actions does
    if cargo clippy --all-targets --all-features -- -D warnings; then
        echo ""
        echo "✅ Clippy check passed! Safe to push."
        cd ..
    else
        echo ""
        echo "❌ Clippy found warnings that would break CI!"
        echo ""
        echo "📋 Common solutions:"
        echo "   1. Run: cargo clippy --fix --allow-dirty"
        echo "   2. Run: cargo fmt"
        echo "   3. Check the output above for specific issues"
        echo ""
        echo "💡 To bypass this hook (not recommended):"
        echo "   git push --no-verify"
        echo ""
        cd ..
        exit 1
    fi
else
    echo "✅ No Rust changes detected, skipping clippy check."
fi

echo "✅ Pre-push checks completed!"
