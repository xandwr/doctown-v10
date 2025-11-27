#!/bin/bash
set -e

# Function to convert GitHub repo URL to zip download URL
github_to_zip() {
    local url="$1"
    local branch="${2:-main}"
    
    # Remove trailing slash
    url="${url%/}"
    
    # Check if it's a GitHub URL
    if [[ "$url" =~ ^https?://github\.com/([^/]+)/([^/]+)/?$ ]]; then
        local owner="${BASH_REMATCH[1]}"
        local repo="${BASH_REMATCH[2]}"
        # Remove .git suffix if present
        repo="${repo%.git}"
        echo "https://github.com/$owner/$repo/archive/refs/heads/$branch.zip"
        return 0
    fi
    
    # If it's already a zip URL, return as-is
    if [[ "$url" =~ \.zip$ ]]; then
        echo "$url"
        return 0
    fi
    
    # Not a recognized URL format
    return 1
}

# Parse arguments to find --repo value
REPO_ARG=""
OUTPUT_ARG=""
OTHER_ARGS=()
SKIP_NEXT=false

for arg in "$@"; do
    if [ "$SKIP_NEXT" = true ]; then
        SKIP_NEXT=false
        continue
    fi
    
    if [[ "$arg" == "--repo" ]] || [[ "$arg" == "-r" ]]; then
        SKIP_NEXT=true
        # Next arg is the repo value
        shift
        REPO_ARG="$1"
    elif [[ "$arg" == --repo=* ]]; then
        REPO_ARG="${arg#*=}"
    elif [[ "$arg" == "--output" ]] || [[ "$arg" == "-o" ]]; then
        SKIP_NEXT=true
        shift
        OUTPUT_ARG="$1"
    elif [[ "$arg" == --output=* ]]; then
        OUTPUT_ARG="${arg#*=}"
    else
        OTHER_ARGS+=("$arg")
    fi
    shift || true
done

# Check if repo looks like a URL
if [[ "$REPO_ARG" =~ ^https?:// ]]; then
    echo "🌐 Detected URL: $REPO_ARG"
    
    # Try to convert GitHub URL to zip
    if ZIP_URL=$(github_to_zip "$REPO_ARG"); then
        echo "📦 Downloading from: $ZIP_URL"
        
        # Create temp directory
        WORK_DIR="/tmp/doctown-work"
        mkdir -p "$WORK_DIR"
        
        # Download the zip
        cd "$WORK_DIR"
        curl -L -o repo.zip "$ZIP_URL"
        
        # Extract
        echo "📂 Extracting archive..."
        unzip -q repo.zip
        
        # Find the extracted directory (GitHub creates a folder like reponame-branch)
        EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "*-*" | head -1)
        
        if [ -z "$EXTRACTED_DIR" ]; then
            echo "❌ ERROR: Could not find extracted directory"
            exit 1
        fi
        
        echo "✓ Repository extracted to: $EXTRACTED_DIR"
        
        # Update repo arg to point to extracted directory
        REPO_ARG="$WORK_DIR/$EXTRACTED_DIR"
        
        # If no output specified, set a sensible default
        if [ -z "$OUTPUT_ARG" ]; then
            BASENAME=$(basename "$EXTRACTED_DIR")
            OUTPUT_ARG="/output/${BASENAME}.docpack"
            mkdir -p /output
        fi
    else
        echo "❌ ERROR: Could not parse URL: $REPO_ARG"
        exit 1
    fi
fi

# Build the final command
CMD_ARGS=("${OTHER_ARGS[@]}")
[ -n "$REPO_ARG" ] && CMD_ARGS+=("--repo" "$REPO_ARG")
[ -n "$OUTPUT_ARG" ] && CMD_ARGS+=("--output" "$OUTPUT_ARG")

# Execute doctown with the processed arguments
doctown "${CMD_ARGS[@]}"
EXIT_CODE=$?

# If successful and local docpacks directory is mounted, copy there too
if [ $EXIT_CODE -eq 0 ] && [ -n "$OUTPUT_ARG" ] && [ -f "$OUTPUT_ARG" ] && [ -d "/local-docpacks" ]; then
    echo ""
    echo "📋 Copying to local docpacks directory..."
    FILENAME=$(basename "$OUTPUT_ARG")
    cp "$OUTPUT_ARG" "/local-docpacks/$FILENAME"
    if [ $? -eq 0 ]; then
        echo "✓ Copied to: ~/.localdoc/docpacks/$FILENAME"
    else
        echo "⚠️  Failed to copy to local docpacks directory"
    fi
fi

exit $EXIT_CODE
