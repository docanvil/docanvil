#!/usr/bin/env bash
set -euo pipefail

KEY=$(bash bin/docs-binary-key.sh)
DEST="$RUNNER_TEMP/docs-binary"
mkdir -p "$DEST"

# Only trust successful push builds from this repository's master CI workflow.
# Expired/missing artifacts and API/download failures all permit a local build.
if artifacts=$(gh api "repos/$GITHUB_REPOSITORY/actions/artifacts?name=master-docanvil-$KEY&per_page=20"); then
  while read -r run_id; do
    [[ -n "$run_id" ]] || continue
    if ! run=$(gh api "repos/$GITHUB_REPOSITORY/actions/runs/$run_id"); then
      continue
    fi
    if ! jq -e '.event == "push" and .head_branch == "master" and .conclusion == "success" and .path == ".github/workflows/ci.yml"' <<< "$run" >/dev/null; then
      continue
    fi
    if gh run download "$run_id" --repo "$GITHUB_REPOSITORY" --name "master-docanvil-$KEY" --dir "$DEST" && [[ -s "$DEST/docanvil" ]]; then
      chmod +x "$DEST/docanvil"
      if "$DEST/docanvil" --version; then
        echo "found=true" >> "$GITHUB_OUTPUT"
        exit 0
      fi
    fi
  done < <(jq -r '.artifacts[] | select(.expired | not) | .workflow_run.id' <<< "$artifacts")
fi

echo "No usable master binary found; building this checkout."
echo "found=false" >> "$GITHUB_OUTPUT"
