#!/usr/bin/env bash

set -euo pipefail

############################################
# Usage
############################################
# ./bin/release.sh 1.2.3
# ./bin/release.sh patch
# ./bin/release.sh minor
# ./bin/release.sh major

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

# --show-current prints the plain branch name even if a tag shares it
CURRENT_BRANCH=$(git branch --show-current)

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
# Ensure CHANGELOG.md documents the release
############################################

CHANGELOG="CHANGELOG.md"
UNRELEASED_HEADING="## [Unreleased]"

if ! grep -qxF "$UNRELEASED_HEADING" "$CHANGELOG"; then
  error "$CHANGELOG has no '$UNRELEASED_HEADING' section. Add one listing the changes in this release."
fi

# Non-blank lines between the Unreleased heading and the next version heading
UNRELEASED_NOTES=$(awk -v h="$UNRELEASED_HEADING" '
  $0 == h { found = 1; next }
  found && /^## \[/ { exit }
  found && NF { print }
' "$CHANGELOG")

# Subheadings alone (e.g. an empty "### Fixed") don't count as documented changes
if ! echo "$UNRELEASED_NOTES" | grep -qv '^#'; then
  error "The '$UNRELEASED_HEADING' section in $CHANGELOG is empty. Document the changes before releasing."
fi

############################################
# Confirm release
############################################

CRATE_NAME=$(grep '^name =' Cargo.toml | head -1 | sed -E 's/name = "(.*)"/\1/')

echo ""
echo "About to open a release PR:"
echo "  Crate:   $CRATE_NAME"
echo "  Current: $CURRENT_VERSION"
echo "  New:     $NEW_VERSION"
echo "  Branch:  release/v$NEW_VERSION"
echo ""
echo "Changelog entries:"
echo "$UNRELEASED_NOTES" | sed 's/^/  /'
echo ""

read -p "Continue? (y/N): " CONFIRM
if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
  echo "Aborted."
  exit 0
fi

############################################
# Create release branch
############################################

echo "Creating branch release/v$NEW_VERSION..."
git checkout -b "release/v$NEW_VERSION"

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
# Move Unreleased changelog entries under the new version
############################################

echo "Updating $CHANGELOG..."
RELEASE_DATE=$(date +%Y-%m-%d)
awk -v h="$UNRELEASED_HEADING" -v v="## [$NEW_VERSION] - $RELEASE_DATE" '
  $0 == h && !done { print h; print ""; print v; done = 1; next }
  { print }
' "$CHANGELOG" > "$CHANGELOG.tmp"
mv "$CHANGELOG.tmp" "$CHANGELOG"

############################################
# Stage and commit version changes
############################################

git add Cargo.toml "$CHANGELOG"

if [ -f Cargo.lock ]; then
  git add Cargo.lock
fi

git commit -m "Release v$NEW_VERSION"

############################################
# Push branch to origin
############################################

echo "Pushing branch to origin..."
git push -u origin "release/v$NEW_VERSION"

############################################
# Open release PR
############################################

echo "Opening release PR..."
gh pr create \
  --title "Release v$NEW_VERSION" \
  --base master \
  --body "## Release v$NEW_VERSION

Bumps version from \`$CURRENT_VERSION\` to \`$NEW_VERSION\`.

### Checklist
- [ ] CHANGELOG.md entries for this version reviewed (moved from [Unreleased] by the release script)
- [ ] Tests pass (see CI)
- [ ] Ready to merge

---
Merging this PR automatically tags \`v$NEW_VERSION\` and runs the release workflow (see \`.github/workflows/tag-release.yml\`). No manual tagging needed."

echo ""
echo "Release PR for v$NEW_VERSION opened. CI will run automatically."
echo ""
echo "Once merged, v$NEW_VERSION is tagged and released automatically 🚀"
