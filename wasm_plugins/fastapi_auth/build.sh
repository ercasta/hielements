#!/bin/bash
# Build script for FastAPI Auth WASM plugin

set -e

echo "Building FastAPI Auth WASM plugin..."

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build the WASM module
echo "Compiling to WASM..."
cargo build --target wasm32-unknown-unknown --release

# Create output directory
mkdir -p ../../lib

# Copy WASM file
WASM_FILE="target/wasm32-unknown-unknown/release/fastapi_auth_wasm.wasm"
OUTPUT_FILE="../../lib/fastapi_auth.wasm"

if [ -f "$WASM_FILE" ]; then
    cp "$WASM_FILE" "$OUTPUT_FILE"
    echo "✓ WASM module built successfully: $OUTPUT_FILE"
    
    # Get file size
    SIZE=$(wc -c < "$OUTPUT_FILE" | xargs)
    SIZE_KB=$((SIZE / 1024))
    echo "  Size: ${SIZE_KB}KB"
    
    # Try to optimize with wasm-opt if available
    if command -v wasm-opt &> /dev/null; then
        echo "Optimizing with wasm-opt..."
        wasm-opt -Oz "$OUTPUT_FILE" -o "${OUTPUT_FILE}.tmp"
        mv "${OUTPUT_FILE}.tmp" "$OUTPUT_FILE"
        
        OPT_SIZE=$(wc -c < "$OUTPUT_FILE" | xargs)
        OPT_SIZE_KB=$((OPT_SIZE / 1024))
        SAVED=$((SIZE_KB - OPT_SIZE_KB))
        echo "✓ Optimized: ${OPT_SIZE_KB}KB (saved ${SAVED}KB)"
    else
        echo "  Note: Install binaryen for smaller WASM files (optional)"
        echo "  Ubuntu/Debian: sudo apt-get install binaryen"
        echo "  macOS: brew install binaryen"
    fi
else
    echo "✗ Build failed: WASM file not found"
    exit 1
fi

echo ""
echo "Done! WASM plugin ready at: $OUTPUT_FILE"
echo ""
echo "To use in Hielements, add to hielements.toml:"
echo ""
echo "[libraries]"
echo "fastapi_auth = { path = \"lib/fastapi_auth.wasm\" }"
