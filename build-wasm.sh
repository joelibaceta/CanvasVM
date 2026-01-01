#!/bin/bash
set -e

echo "Building Canvas VM WebAssembly package..."

# Build with wasm-pack
wasm-pack build crates/canvas_wasm --target web --out-dir ../../docs/pkg

# Update package.json with correct name
cd docs/pkg
if [ -f package.json ]; then
    # Use jq if available, otherwise use sed
    if command -v jq &> /dev/null; then
        # Use jq for safe JSON manipulation
        jq '.name = "canvas-vm-wasm" | .homepage = "https://canvasvm.com" | .files += ["canvas_wasm_bg.wasm.d.ts"] | .keywords += ["webassembly", "wasm", "visual-programming", "runtime"]' package.json > package.json.tmp
        mv package.json.tmp package.json
    else
        # Fallback to sed (less safe but works)
        sed -i.bak 's/"name": "canvas_wasm"/"name": "canvas-vm-wasm"/' package.json
        sed -i.bak 's|"homepage": "https://github.com/joelibaceta/CanvasVM"|"homepage": "https://canvasvm.com"|' package.json
        rm package.json.bak
    fi
    echo "✓ Updated package name to canvas-vm-wasm"
else
    echo "✗ package.json not found"
    exit 1
fi

cd ../..
echo "✓ Build complete! Package ready at docs/pkg/"
echo ""
echo "To publish to npm:"
echo "  cd docs/pkg"
echo "  npm publish --access public"
