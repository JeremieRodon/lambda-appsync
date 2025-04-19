#!/usr/bin/env bash
set -e

HOOK_DIR=.hooks
GIT_DIR=.git/hooks

echo "Installing git hooks..."

# Create symbolic links for all hooks
for hook in "$HOOK_DIR"/*; do
    if [ -f "$hook" ]; then
        hook_name=$(basename "$hook")
        ln -sf "../../$HOOK_DIR/$hook_name" "$GIT_DIR/$hook_name"
        echo "Installed $hook_name hook"
    fi
done

echo "Git hooks installed successfully!"
