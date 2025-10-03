#!/bin/bash
# Install git hooks for coverage tracking

HOOK_DIR=".git/hooks"
PRE_PUSH_HOOK="$HOOK_DIR/pre-push"

install_hooks() {
    echo "Installing git hooks..."

    # Create hooks directory if it doesn't exist
    mkdir -p "$HOOK_DIR"

    # Install pre-push hook
    if [ -f "$PRE_PUSH_HOOK" ]; then
        echo "⚠️  Pre-push hook already exists at $PRE_PUSH_HOOK"
        echo ""
        read -p "Overwrite existing hook? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted. To manually install, copy:"
            echo "  cp scripts/pre-push-coverage .git/hooks/pre-push"
            exit 0
        fi
    fi

    # Copy hook
    cp scripts/pre-push-coverage "$PRE_PUSH_HOOK"
    chmod +x "$PRE_PUSH_HOOK"

    echo "✅ Pre-push hook installed at $PRE_PUSH_HOOK"
    echo ""
    echo "The hook will:"
    echo "  • Run coverage report generation before each push"
    echo "  • Update COVERAGE_REPORT.md if coverage changed"
    echo "  • Require you to commit the updated report before pushing"
    echo ""
    echo "To bypass the hook on a specific push: git push --no-verify"
    echo "To uninstall: ./scripts/install-git-hooks.sh uninstall"
}

uninstall_hooks() {
    echo "Uninstalling git hooks..."

    if [ -f "$PRE_PUSH_HOOK" ]; then
        rm "$PRE_PUSH_HOOK"
        echo "✅ Pre-push hook removed"
    else
        echo "ℹ️  No pre-push hook found"
    fi
}

case "${1:-install}" in
    install)
        install_hooks
        ;;
    uninstall)
        uninstall_hooks
        ;;
    *)
        echo "Usage: $0 [install|uninstall]"
        echo ""
        echo "  install   - Install git hooks (default)"
        echo "  uninstall - Remove git hooks"
        exit 1
        ;;
esac
