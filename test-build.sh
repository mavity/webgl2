# Run with: bash build-and-test.sh or sh build-and-test.sh

set -e  # Exit on error

echo "=== Initializing submodules ==="
git submodule update --init --depth 2 --single-branch || {
    echo "Warning: Submodule initialization had issues"
}

echo ""
echo "=== Building project ==="
npm run build

echo ""
echo "=== Running tests ==="
npm test

echo ""
echo "=== SUCCESS ===" 
