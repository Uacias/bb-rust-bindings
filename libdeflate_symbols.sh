#!/bin/bash

LIBDEFLATE="/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp/build/_deps/libdeflate-build/libdeflate.a"

echo "Checking symbols in libdeflate.a..."
echo "===================================="

if [ -f "$LIBDEFLATE" ]; then
    echo "File exists: $LIBDEFLATE"
    echo -e "\nChecking for required symbols:"
    nm "$LIBDEFLATE" 2>/dev/null | grep -E "(libdeflate_alloc_decompressor|libdeflate_gzip_decompress|libdeflate_free_decompressor)" | head -20
    
    echo -e "\nAll defined symbols (T) in libdeflate:"
    nm "$LIBDEFLATE" 2>/dev/null | grep " T " | head -20
else
    echo "libdeflate.a not found at: $LIBDEFLATE"
fi

echo -e "\nChecking if we need to link in a specific order..."
echo "Library dependencies in barretenberg:"
ldd /home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp/build/lib/libbarretenberg.a 2>&1 || echo "Static library - no ldd output"

echo -e "\nTrying to find libdeflate in system:"
pkg-config --libs libdeflate 2>/dev/null || echo "No pkg-config for libdeflate"

echo -e "\nChecking link order - libbarretenberg should come BEFORE libdeflate:"
echo "Current order should be: -lbarretenberg -ldeflate"