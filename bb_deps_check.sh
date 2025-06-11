#!/bin/bash

BARRETENBERG_BUILD="/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp/build"

echo "Checking Barretenberg build directory structure..."
echo "================================================"

echo -e "\nChecking main lib directory:"
ls -la "$BARRETENBERG_BUILD/lib/" 2>/dev/null || echo "lib/ directory not found"

echo -e "\nChecking for libdeflate:"
find "$BARRETENBERG_BUILD" -name "*deflate*" -type f 2>/dev/null | head -20

echo -e "\nChecking _deps directory:"
ls -la "$BARRETENBERG_BUILD/_deps/" 2>/dev/null | grep -E "(deflate|libdeflate)" || echo "No deflate in _deps"

echo -e "\nChecking for all .a files:"
find "$BARRETENBERG_BUILD" -name "*.a" -type f 2>/dev/null | head -20

echo -e "\nChecking libbarretenberg.a dependencies:"
if [ -f "$BARRETENBERG_BUILD/lib/libbarretenberg.a" ]; then
    echo "Using nm to check undefined symbols in libbarretenberg.a:"
    nm -u "$BARRETENBERG_BUILD/lib/libbarretenberg.a" 2>/dev/null | grep -E "(deflate|compress)" | head -10
fi

echo -e "\nChecking system libdeflate:"
ldconfig -p 2>/dev/null | grep deflate || echo "No system libdeflate found"

echo -e "\nChecking if libdeflate is bundled in libbarretenberg.a:"
if [ -f "$BARRETENBERG_BUILD/lib/libbarretenberg.a" ]; then
    ar t "$BARRETENBERG_BUILD/lib/libbarretenberg.a" 2>/dev/null | grep -i deflate | head -5 || echo "No deflate objects found in archive"
fi