#!/usr/bin/env bash

set -euo pipefail

############################################
# Usage
############################################
# ./release.sh 1.2.3
# ./release.sh patch
# ./release.sh minor
# ./release.sh major

############################################
# Helpers
############################################

error() {
  echo "Error: $1"
  exit 1
}

############################################
# Validate input
############################################

if [ $# -ne 1 ]; then
  echo "Usage: $0 <version|patch|minor|major>"
  exit 1
fi

ARG="$1"

############################################
# Extract current version
############################################

CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed -E 's/version = "(.*)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
  error "Could not determine current version from Cargo.toml"
fi

echo "Current version: $CURRENT_VERSION"

############################################
# SemVer parsing
############################################

SEMVER_REGEX='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'

increment_version() {
  IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
  PATCH="${PATCH%%-*}"  # strip prerelease

  case "$1" in
    patch)
      PATCH=$((PATCH + 1))
      ;;
    minor)
      MINOR=$((MINOR + 1))
      PATCH=0
      ;;
    major)
      MAJOR=$((MAJOR + 1))
      MINOR=0
      PATCH=0
      ;;
    *)
      error "Invalid increment type"
      ;;
  esac

  echo "$MAJOR.$MINOR.$PATCH"
}

############################################
# Determine new version
############################################

if [[ "$ARG" == "patch" || "$ARG" == "minor" || "$ARG" == "major" ]]; then
  NEW_VERSION=$(increment_version "$ARG")
else
  NEW_VERSION="$ARG"
fi

############################################
# Validate SemVer format
############################################

if [[ ! "$NEW_VERSION" =~ $SEMVER_REGEX ]]; then
  error "Version must follow SemVer (e.g. 1.2.3 or 1.2.3-alpha.1)"
fi

############################################
# Ensure new version > current version
############################################

if [ "$NEW_VERSION" = "$CURRENT_VERSION" ]; then
  error "New version must differ from current version"
fi

############################################
# Ensure clean working directory
############################################

if ! git diff-index --quiet HEAD --; then
  error "Working directory is not clean. Commit or stash changes first."
fi

############################################
# Ensure correct branch (master)
############################################

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

if [ "$CURRENT_BRANCH" != "master" ]; then
  error "Releases must be made from master branch (current: $CURRENT_BRANCH)"
fi

############################################
# Ensure tag does not already exist
############################################

if git rev-parse "v$NEW_VERSION" >/dev/null 2>&1; then
  error "Git tag v$NEW_VERSION already exists"
fi

############################################
# Check crates.io for existing version
############################################

CRATE_NAME=$(grep '^name =' Cargo.toml | head -1 | sed -E 's/name = "(.*)"/\1/')

if cargo search "$CRATE_NAME" | grep -q "$NEW_VERSION"; then
  error "Version $NEW_VERSION already appears to exist on crates.io"
fi

############################################
# Confirm release
############################################

echo ""
echo "About to release:"
echo "  Crate:   $CRATE_NAME"
echo "  Current: $CURRENT_VERSION"
echo "  New:     $NEW_VERSION"
echo ""

read -p "Continue with release? (y/N): " CONFIRM
if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
  echo "Aborted."
  exit 0
fi

############################################
# Update Cargo.toml (macOS safe)
############################################

echo "Updating Cargo.toml..."
sed -i '' -E "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

############################################
# Ensure lockfile is up to date
############################################

echo "Updating Cargo.lock..."
cargo check > /dev/null

############################################
# Stage version changes
############################################

git add Cargo.toml

if [ -f Cargo.lock ]; then
  git add Cargo.lock
fi

############################################
# Commit version bump
############################################

git commit -m "Release v$NEW_VERSION"

############################################
# Run tests
############################################

echo "Running tests..."
cargo test

############################################
# Dry run publish
############################################

echo "Running cargo publish --dry-run..."
cargo publish --dry-run

############################################
# Publish
############################################

echo "Publishing to crates.io..."
cargo publish

echo "Publish successful."

############################################
# Create annotated tag
############################################

git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

############################################
# Push commit + tag
############################################

echo "Pushing to origin..."
git push origin master
git push origin "v$NEW_VERSION"

echo ""
echo "Release v$NEW_VERSION complete."