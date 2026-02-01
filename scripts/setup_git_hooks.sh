#!/bin/bash

# Setup script to install git hooks from scripts/git-hooks/ to .git/hooks/
# This allows git hooks to be tracked in version control and shared with the team

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOKS_SOURCE="${REPO_ROOT}/scripts/git-hooks"
HOOKS_TARGET="${REPO_ROOT}/.git/hooks"

if [ ! -d "${HOOKS_TARGET}" ]; then
    echo "Error: .git/hooks directory does not exist. Are you in a git repository?" >&2
    exit 1
fi

echo "Setting up git hooks..."

# Install pre-push hook
PRE_PUSH_SOURCE="${HOOKS_SOURCE}/pre-push"
PRE_PUSH_TARGET="${HOOKS_TARGET}/pre-push"

if [ ! -f "${PRE_PUSH_SOURCE}" ]; then
    echo "Error: pre-push hook not found at ${PRE_PUSH_SOURCE}" >&2
    exit 1
fi

# Copy the hook
cp "${PRE_PUSH_SOURCE}" "${PRE_PUSH_TARGET}"
chmod +x "${PRE_PUSH_TARGET}"

echo "✓ Git hooks installed successfully!"
echo "  Pre-push hook: ${PRE_PUSH_TARGET}"

