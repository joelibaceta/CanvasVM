#!/bin/bash

# Script para compilar CanvasVM a WebAssembly

set -e

echo "🎨 Building CanvasVM for WebAssembly..."

# Verificar que wasm-pack esté instalado
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack no está instalado"
    echo "Instalar con: cargo install wasm-pack"
    exit 1
fi

cd "$(dirname "$0")/.."

TARGET=${1:-web}

echo "📦 Compilando para target: $TARGET"

case $TARGET in
  web)
    wasm-pack build --target web --out-dir ../../pkg crates/canvas_wasm
    ;;
  nodejs)
    wasm-pack build --target nodejs --out-dir ../../pkg crates/canvas_wasm
    ;;
  bundler)
    wasm-pack build --target bundler --out-dir ../../pkg crates/canvas_wasm
    ;;
  *)
    echo "❌ Target no válido: $TARGET"
    echo "Uso: $0 [web|nodejs|bundler]"
    exit 1
    ;;
esac

echo "✅ Build completado!"
echo "📁 Output en: pkg/"
echo ""
echo "Para probar en el navegador:"
echo "  1. cd pkg"
echo "  2. python3 -m http.server 8000"
echo "  3. Abrir http://localhost:8000"
