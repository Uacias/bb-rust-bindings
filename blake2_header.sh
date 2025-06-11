#!/bin/bash

BARRETENBERG_PATH="/home/uacias/dev/visoft/aztec/aztec-packages/barretenberg/cpp"

echo "Looking for blake2s C binding header..."
echo "======================================="

# Find blake2s headers
find "$BARRETENBERG_PATH/src" -name "*blake2s*" -name "*.hpp" -o -name "*.h" | while read -r file; do
    if [[ "$file" == *"c_bind"* ]]; then
        echo -e "\n=== $file ==="
        cat "$file"
    fi
done

echo -e "\n\nLooking for blake2s implementation..."
echo "======================================"

# Check for implementation details
find "$BARRETENBERG_PATH/src" -name "*blake2s*" -name "*.cpp" | while read -r file; do
    if [[ "$file" == *"c_bind"* ]]; then
        echo -e "\n=== $file ==="
        # Show first 50 lines which usually contain the important bits
        head -50 "$file"
    fi
done

echo -e "\n\nChecking serialization format..."
echo "================================"

# Look for serialization utilities
grep -r "serialize.*buffer\|read_.*buffer" "$BARRETENBERG_PATH/src/barretenberg/common/" | head -10