#!/usr/bin/env bash
# Build script for doctown Docker image

set -e

echo "════════════════════════════════════════════"
echo " Building doctown Docker image"
echo "════════════════════════════════════════════"
echo ""

# Build the Docker image
docker build -t doctown:latest .

echo ""
echo "✅ Build complete!"
echo ""
echo "To run the container:"
echo "  docker run --rm -v \$(pwd):/workspace doctown:latest build --repo /workspace --output /workspace/output.docpack"
echo ""
echo "With GPU support:"
echo "  docker run --rm --gpus all -v \$(pwd):/workspace doctown:latest build --repo /workspace --output /workspace/output.docpack"
echo ""
