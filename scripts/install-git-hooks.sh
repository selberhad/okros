#!/bin/bash
# Install git hooks for coverage tracking

HOOK_DIR=".git/hooks"
PRE_COMMIT_HOOK="$HOOK_DIR/pre-commit"

install_hooks() {
    echo "Installing git hooks..."

    # Create hooks directory if it doesn't exist
    mkdir -p "$HOOK_DIR"

    # Install pre-commit hook
    if [ -f "$PRE_COMMIT_HOOK" ]; then
        echo "⚠️  Pre-commit hook already exists at $PRE_COMMIT_HOOK"
        echo ""
        read -p "Overwrite existing hook? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted. To manually install, copy:"
            echo "  cp scripts/pre-commit-coverage .git/hooks/pre-commit"
            exit 0
        fi
    fi

    # Copy hook
    cp scripts/pre-commit-coverage "$PRE_COMMIT_HOOK"
    chmod +x "$PRE_COMMIT_HOOK"

    echo "✅ Pre-commit hook installed at $PRE_COMMIT_HOOK"
    echo ""
    echo "The hook will:"
    echo "  • Run coverage report generation before each commit"
    echo "  • Update COVERAGE_REPORT.md if coverage changed"
    echo "  • Auto-stage the updated report (no manual intervention!)"
    echo ""
    echo "To bypass the hook on a specific commit: git commit --no-verify"
    echo "To uninstall: ./scripts/install-git-hooks.sh uninstall"
}

uninstall_hooks() {
    echo "Uninstalling git hooks..."

    if [ -f "$PRE_COMMIT_HOOK" ]; then
        rm "$PRE_COMMIT_HOOK"
        echo "✅ Pre-commit hook removed"
    else
        echo "ℹ️  No pre-commit hook found"
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
