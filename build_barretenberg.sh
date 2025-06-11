#!/bin/bash

# Skrypt do buildowania Barretenberg z właściwymi flagami dla Rust bindingów
# Umieść w głównym katalogu barretenberg-rust-bindings

set -e  # Exit on any error

BARRETENBERG_PATH="$HOME/dev/visoft/aztec/aztec-packages/barretenberg/cpp"

echo "🔨 Building Barretenberg with Rust-friendly flags..."

# Sprawdź czy katalog istnieje
if [ ! -d "$BARRETENBERG_PATH" ]; then
    echo "❌ Error: Barretenberg path not found: $BARRETENBERG_PATH"
    echo "Please update BARRETENBERG_PATH in this script"
    exit 1
fi

cd "$BARRETENBERG_PATH"

echo "📁 Current directory: $(pwd)"

# Usuń stary build (opcjonalnie)
if [ "$1" = "--clean" ]; then
    echo "🧹 Cleaning old build directory..."
    rm -rf build
fi

echo "⚙️  Configuring CMake with Tracy disabled..."
cmake --preset=default -DTRACY_ENABLE=OFF

echo "🔨 Building bb target..."
cmake --build build --target bb

echo "📋 Copying headers..."
./copy-headers.sh build/include

echo "✅ Barretenberg build completed successfully!"
echo "📍 Build location: $BARRETENBERG_PATH/build"
echo "📍 Headers location: $BARRETENBERG_PATH/build/include"

# Sprawdź czy biblioteka została utworzona
LIB_PATH="$BARRETENBERG_PATH/build/lib/libbarretenberg.a"
if [ -f "$LIB_PATH" ]; then
    echo "✅ Library found: $LIB_PATH"
    echo "📊 Library size: $(ls -lh "$LIB_PATH" | awk '{print $5}')"
else
    echo "⚠️  Warning: Library not found at expected location: $LIB_PATH"
fi

echo ""
echo "🚀 Ready to build Rust bindings with:"
echo "   cargo build"