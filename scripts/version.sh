#!/bin/bash

# Version management utility for hsipc
# Usage: ./scripts/version.sh [show|set|bump]

set -e

# Get current version
get_current_version() {
    grep -E '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Show current version
show_version() {
    local current=$(get_current_version)
    echo "📋 Current version: $current"
    
    # Show what next versions would be
    IFS='.' read -r -a version_parts <<< "$current"
    major=${version_parts[0]}
    minor=${version_parts[1]}
    patch=${version_parts[2]}
    
    echo "🔄 Next versions:"
    echo "  patch: $major.$minor.$((patch + 1))"
    echo "  minor: $major.$((minor + 1)).0"
    echo "  major: $((major + 1)).0.0"
}

# Set specific version
set_version() {
    local new_version=$1
    
    if [[ ! $new_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format X.Y.Z (e.g., 0.1.1)"
        exit 1
    fi
    
    echo "📝 Setting version to $new_version"
    sed -i "s/version = \".*\"/version = \"$new_version\"/" Cargo.toml
    
    # Verify the change
    local current=$(get_current_version)
    if [ "$current" = "$new_version" ]; then
        echo "✅ Version updated successfully: $new_version"
    else
        echo "❌ Failed to update version"
        exit 1
    fi
}

# Bump version
bump_version() {
    local bump_type=$1
    local current=$(get_current_version)
    
    IFS='.' read -r -a version_parts <<< "$current"
    major=${version_parts[0]}
    minor=${version_parts[1]}
    patch=${version_parts[2]}
    
    case $bump_type in
        patch)
            new_version="$major.$minor.$((patch + 1))"
            ;;
        minor)
            new_version="$major.$((minor + 1)).0"
            ;;
        major)
            new_version="$((major + 1)).0.0"
            ;;
        *)
            echo "Error: Invalid bump type. Use patch, minor, or major"
            exit 1
            ;;
    esac
    
    echo "📝 Bumping version from $current to $new_version"
    set_version "$new_version"
}

# Main command handling
case "${1:-show}" in
    show)
        show_version
        ;;
    set)
        if [ $# -ne 2 ]; then
            echo "Usage: $0 set <version>"
            echo "Example: $0 set 0.1.1"
            exit 1
        fi
        set_version "$2"
        ;;
    bump)
        if [ $# -ne 2 ]; then
            echo "Usage: $0 bump <patch|minor|major>"
            echo "Examples:"
            echo "  $0 bump patch"
            echo "  $0 bump minor"
            echo "  $0 bump major"
            exit 1
        fi
        bump_version "$2"
        ;;
    *)
        echo "Usage: $0 [show|set|bump]"
        echo ""
        echo "Commands:"
        echo "  show                 Show current version and next options"
        echo "  set <version>        Set specific version"
        echo "  bump <patch|minor|major>  Bump version"
        echo ""
        echo "Examples:"
        echo "  $0 show"
        echo "  $0 set 0.1.1"
        echo "  $0 bump patch"
        exit 1
        ;;
esac