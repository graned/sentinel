#!/bin/bash

set -e

echo "🧪 Running tests for sentinel-auth..."

# Unit tests
echo "📚 Running unit tests..."
cargo test

# Integration tests
echo "🔗 Running integration tests..."
cargo test --tests

echo "✅ All tests passed!"
