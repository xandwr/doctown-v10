#!/bin/bash
# Build a docpack from a GitHub repository and save it locally

set -e

# Check if repo URL was provided
if [ -z "$1" ]; then
    echo "Usage: $0 <github-repo-url> [additional-options]"
    echo ""
    echo "Examples:"
    echo "  $0 https://github.com/xandwr/localdoc"
    echo "  $0 https://github.com/owner/repo --skip-embeddings"
    echo ""
    exit 1
fi

REPO_URL="$1"
shift  # Remove first argument, keep the rest

# Create local docpacks directory
LOCAL_DIR="$HOME/.localdoc/docpacks"
mkdir -p "$LOCAL_DIR"

echo "📦 Building docpack from: $REPO_URL"
echo "💾 Output directory: $LOCAL_DIR"
echo ""

# Run docker with local directory mounted
docker run -v "$LOCAL_DIR:/output" doctown build --repo "$REPO_URL" "$@"

echo ""
echo "✅ Docpack saved to: $LOCAL_DIR"
echo "📂 List files: ls -lh $LOCAL_DIR"
