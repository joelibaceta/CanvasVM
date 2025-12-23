#!/bin/bash
# Build script for CanvasVM

set -e

echo "🔨 Building WASM..."
wasm-pack build --target web --out-dir ../../docs/pkg crates/canvas_wasm

echo "🗑️  Removing unnecessary .gitignore from pkg..."
rm -f docs/pkg/.gitignore

echo "✅ Build complete! Files are in docs/pkg/"
echo ""
echo "To test locally:"
echo "  cd docs && python3 -m http.server 8080"
echo ""
echo "To deploy, commit and push:"
echo "  git add -A && git commit -m 'Update WASM build' && git push"
