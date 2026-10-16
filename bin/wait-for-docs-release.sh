#!/usr/bin/env bash
set -euo pipefail

VERSION=$(sed -nE 's/^current = "([^"]+)"/\1/p' docs/docanvil.toml)
if [[ -z "$VERSION" ]]; then
  echo "::error::Cannot determine the current docs version"
  exit 1
fi

# Allow the merge-triggered tag and release dispatch to reach GitHub's API.
# Once the run is found, watch its actual completion rather than sleeping for
# a fixed release duration. Match the tag's commit as well as its name.
for ((attempt = 0; attempt < 60; attempt++)); do
  if git show-ref --verify --quiet "refs/tags/$VERSION"; then
    commit=$(git rev-parse "refs/tags/$VERSION^{commit}")
    runs=$(gh run list --repo "$GITHUB_REPOSITORY" --workflow release.yml \
      --branch "$VERSION" --limit 20 --json databaseId,headSha)
    run_id=$(jq -r --arg sha "$commit" '[.[] | select(.headSha == $sha)][0].databaseId // empty' <<< "$runs")
    if [[ -n "$run_id" ]]; then
      echo "Waiting for release $VERSION (run $run_id)."
      gh run watch "$run_id" --repo "$GITHUB_REPOSITORY" --exit-status
      git fetch origin --tags
      exit 0
    fi
  fi
  echo "Waiting for release $VERSION to start ($((attempt + 1))/60)..."
  sleep 5
  git fetch origin --tags
done

echo "::error::No matching release run appeared for $VERSION; check tag-release.yml"
exit 1
