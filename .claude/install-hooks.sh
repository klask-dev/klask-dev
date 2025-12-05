#!/bin/bash
# Install Git hooks for Klask development
# This script sets up pre-commit, pre-push, and post-code-change hooks

set -e

HOOK_DIR=".git/hooks"
CLAUDE_HOOK_DIR=".claude/hooks"

echo "📦 Installing Git hooks for Klask..."
echo ""

# Check if we're in the right directory
if [ ! -d "$HOOK_DIR" ] || [ ! -d "$CLAUDE_HOOK_DIR" ]; then
    echo "❌ Error: This script must be run from the root of the klask-dev repository"
    exit 1
fi

# Install pre-commit hook
echo "📝 Installing pre-commit hook..."
cat > "$HOOK_DIR/pre-commit" << 'EOF'
#!/bin/bash
exec ./.claude/hooks/pre-commit.sh
EOF
chmod +x "$HOOK_DIR/pre-commit"
echo "   ✅ pre-commit hook installed"

# Install pre-push hook
echo "📝 Installing pre-push hook..."
cat > "$HOOK_DIR/pre-push" << 'EOF'
#!/bin/bash
exec ./.claude/hooks/pre-push.sh
EOF
chmod +x "$HOOK_DIR/pre-push"
echo "   ✅ pre-push hook installed"

# Verify hooks are executable
if [ -x "$CLAUDE_HOOK_DIR/pre-commit.sh" ] && [ -x "$CLAUDE_HOOK_DIR/pre-push.sh" ]; then
    echo ""
    echo "✅ All Git hooks installed successfully!"
    echo ""
    echo "🎯 What's now active:"
    echo "   • pre-commit: Runs before every commit"
    echo "     └─ Formats code, runs clippy, tests, and linting"
    echo ""
    echo "   • pre-push: Runs before every push"
    echo "     └─ Runs cargo clippy -- -D warnings"
    echo "     └─ Prevents pushing code that fails CI"
    echo ""
    echo "💡 To bypass hooks (not recommended):"
    echo "   git commit --no-verify"
    echo "   git push --no-verify"
    echo ""
else
    echo "❌ Error: Hook scripts are not executable"
    echo "   Run: chmod +x $CLAUDE_HOOK_DIR/*.sh"
    exit 1
fi
