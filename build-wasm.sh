#!/usr/bin/env bash
# build-wasm.sh - build the wasm2rdf crate for use in JS apps

set -euo pipefail

# Configurable variables
CRATE_NAME="wasm2rdf"
TARGET_DIR="target/wasm32-unknown-unknown/release"
WASM_FILE="$TARGET_DIR/${CRATE_NAME}.wasm"

# Prompt for target
echo "Choose the wasm-bindgen target (web, bundler, nodejs):"
read -r TARGET
if [[ ! "$TARGET" =~ ^(web|bundler|nodejs)$ ]]; then
  echo "Invalid target. Defaulting to bundler."
  TARGET="bundler"
fi
OUT_DIR="pkg-${TARGET}"

# Prompt for copy destination
echo "Enter the folder to copy the package to (default: ../rdf-explorer/lib/${CRATE_NAME}):"
read -r COPY_TO_INPUT
COPY_TO="${COPY_TO_INPUT:-../rdf-explorer/lib/${CRATE_NAME}}"

# Set environment variable for getrandom to use wasm_js backend
export RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\""

echo "Building ${CRATE_NAME} for wasm32-unknown-unknown..."
cargo build --lib --release --target wasm32-unknown-unknown

if [ ! -f "$WASM_FILE" ]; then
  echo "Expected wasm file not found: $WASM_FILE"
  exit 1
fi

# Ensure wasm-bindgen is installed
if ! command -v wasm-bindgen &> /dev/null; then
  echo "wasm-bindgen not found, installing wasm-bindgen-cli..."
  cargo install wasm-bindgen-cli || true
fi

echo "Running wasm-bindgen (${TARGET} target)..."
rm -rf "$OUT_DIR"
wasm-bindgen "$WASM_FILE" --out-dir "$OUT_DIR" --target "$TARGET"

# Optional: optimize with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
  echo "Optimizing wasm with wasm-opt..."
  OPT_WASM="$OUT_DIR/${CRATE_NAME}_bg.wasm"
  if [ -f "$OPT_WASM" ]; then
    wasm-opt -Oz -o "$OPT_WASM" "$OPT_WASM" || true
  else
    # try common alternative name
    ALT_WASM="$OUT_DIR/${CRATE_NAME}.wasm"
    if [ -f "$ALT_WASM" ]; then
      wasm-opt -Oz -o "$ALT_WASM" "$ALT_WASM" || true
    fi
  fi
fi

echo "Preparing copy to $COPY_TO..."
mkdir -p "$COPY_TO"
cp -r "$OUT_DIR/"* "$COPY_TO/"
echo "Copied ${TARGET} package to $COPY_TO"

echo "Build complete. Package: $OUT_DIR"
echo "If you need a different target (nodejs, web, bundler) run the script again with a different choice."
