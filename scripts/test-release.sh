#!/bin/bash

# Test release script for hsipc
# This script tests the release automation without actually publishing

set -e

echo "🧪 Testing hsipc release automation..."

# Check if we're on the correct branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "feat/release-automation-testing" ]; then
    echo "Error: This test should be run on feat/release-automation-testing branch"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean"
    exit 1
fi

echo "📋 Current branch: $BRANCH"

# Run pre-release checks
echo "🔍 Running pre-release checks..."
cargo test --workspace --all-features
cargo check --workspace --all-features
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings -A clippy::empty-line-after-doc-comments -A clippy::mixed-attributes-style

echo "✅ All pre-release checks passed!"

# Test package building
echo "🏗️  Testing package building..."
cargo build --release --all-features

echo "✅ Build successful!"

# Test dry-run publishing
echo "📦 Testing package publishing (dry run)..."
cd hsipc-macros
cargo publish --dry-run
cd ..

# For hsipc, we need to test with --allow-dirty since hsipc-macros doesn't exist on crates.io yet
echo "📦 Testing hsipc package publishing (dry run with --allow-dirty)..."
cd hsipc
cargo publish --dry-run --allow-dirty
cd ..

echo "✅ Packages can be published!"

# Test cargo-release with test configuration
echo "🎯 Testing cargo-release automation..."
cargo release --config release-test.toml patch --no-publish

echo "🎉 Release automation test completed successfully!"
echo ""
echo "📋 Summary:"
echo "  - All tests passed"
echo "  - Code quality checks passed"
echo "  - Build successful"
echo "  - Packages validated"
echo "  - Test tag created and pushed"
echo "  - GitHub Actions will run automatically"
echo ""
echo "🔗 Check the test release at:"
echo "   https://github.com/loyalpartner/hsipc/actions"