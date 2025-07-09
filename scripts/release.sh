#!/bin/bash

# Release script for hsipc
# Usage: ./scripts/release.sh [patch|minor|major|version]
# Examples: 
#   ./scripts/release.sh patch      # 0.1.0 → 0.1.1
#   ./scripts/release.sh minor      # 0.1.0 → 0.2.0
#   ./scripts/release.sh major      # 0.1.0 → 1.0.0
#   ./scripts/release.sh 0.1.1      # Set specific version

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <patch|minor|major|version>"
    echo "Examples:"
    echo "  $0 patch      # 0.1.0 → 0.1.1"
    echo "  $0 minor      # 0.1.0 → 0.2.0"
    echo "  $0 major      # 0.1.0 → 1.0.0"
    echo "  $0 0.1.1      # Set specific version"
    exit 1
fi

RELEASE_TYPE=$1

# Get current version
CURRENT_VERSION=$(grep -E '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "📋 Current version: $CURRENT_VERSION"

# Calculate new version
if [[ $RELEASE_TYPE =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    VERSION=$RELEASE_TYPE
else
    IFS='.' read -r -a version_parts <<< "$CURRENT_VERSION"
    major=${version_parts[0]}
    minor=${version_parts[1]}
    patch=${version_parts[2]}
    
    case $RELEASE_TYPE in
        patch)
            VERSION="$major.$minor.$((patch + 1))"
            ;;
        minor)
            VERSION="$major.$((minor + 1)).0"
            ;;
        major)
            VERSION="$((major + 1)).0.0"
            ;;
        *)
            echo "Error: Invalid release type. Use patch, minor, major, or specific version"
            exit 1
            ;;
    esac
fi

echo "🚀 Preparing release $VERSION"

# Check if we're on master branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "master" ]; then
    echo "Error: Must be on master branch to release"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean"
    exit 1
fi

# Update version in workspace Cargo.toml
echo "📝 Updating version in Cargo.toml"
sed -i "s/version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update CHANGELOG.md
echo "📝 Updating CHANGELOG.md"
TODAY=$(date +%Y-%m-%d)
sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$VERSION] - $TODAY/" CHANGELOG.md

# Run tests to make sure everything works
echo "🧪 Running tests"
cargo test --all-features

# Check if packages can be published
echo "🔍 Checking package validity"
cd hsipc-macros && cargo check --release
cd ../hsipc && cargo check --release
cd ..

# Commit version bump
echo "📤 Committing version bump"
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $VERSION"

# Create tag
echo "🏷️  Creating tag v$VERSION"
git tag -a "v$VERSION" -m "Release version $VERSION"

# Push changes and tag
echo "🚀 Pushing to GitHub"
git push origin master
git push origin "v$VERSION"

echo "✅ Release $VERSION created successfully!"
echo "📦 GitHub Actions will automatically publish to crates.io"
echo "🔗 Check progress at: https://github.com/loyalpartner/hsipc/actions"