#!/bin/bash
# Post-code-change hook for Klask
# Automatically runs relevant tests after code changes

# This hook is triggered automatically by Claude Code after file modifications

CHANGED_FILE="$1"

# Skip if no file specified
if [ -z "$CHANGED_FILE" ]; then
    exit 0
fi

echo "🔄 Post-code-change check for: $CHANGED_FILE"

# Rust backend changes
if [[ "$CHANGED_FILE" == klask-rs/* ]]; then
    # Don't run for test files
    if [[ "$CHANGED_FILE" == *"_test.rs"* ]] || [[ "$CHANGED_FILE" == *"/tests/"* ]]; then
        echo "  ℹ️  Test file changed, skipping auto-test"
        exit 0
    fi

    echo "  🦀 Running Rust tests..."
    cd klask-rs

    # Try to run only related tests first
    MODULE_NAME=$(echo "$CHANGED_FILE" | sed 's|klask-rs/src/||' | sed 's|\.rs$||' | sed 's|/|::|g')

    # Run module tests if they exist
    if cargo test --quiet "$MODULE_NAME" 2>/dev/null; then
        echo "  ✅ Related tests passed"
    else
        # Fallback to all tests
        echo "  ⚠️  Running all tests..."
        if cargo test --quiet; then
            echo "  ✅ All tests passed"
        else
            echo "  ❌ Tests failed - please review changes"
            cd ..
            exit 1
        fi
    fi
    cd ..
fi

# React frontend changes
if [[ "$CHANGED_FILE" == klask-react/* ]]; then
    # Don't run for test files
    if [[ "$CHANGED_FILE" == *".test."* ]] || [[ "$CHANGED_FILE" == *"__tests__"* ]]; then
        echo "  ℹ️  Test file changed, skipping auto-test"
        exit 0
    fi

    echo "  ⚛️  Running React tests..."
    cd klask-react

    # Try to run related tests
    TEST_FILE=$(echo "$CHANGED_FILE" | sed 's|klask-react/src/||' | sed 's|\.tsx\?$|.test.ts|')
    TEST_DIR=$(dirname "$TEST_FILE")

    # Run related tests if they exist
    if [ -f "src/$TEST_FILE" ] || [ -f "src/$TEST_DIR/__tests__" ]; then
        if npm test -- --run "src/$TEST_DIR" 2>/dev/null; then
            echo "  ✅ Related tests passed"
        else
            echo "  ⚠️  No related tests found or tests failed"
        fi
    else
        echo "  ℹ️  No related tests found"
    fi
    cd ..
fi

echo "✅ Post-code-change check complete"
