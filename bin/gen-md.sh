#!/usr/bin/env bash

# ================================
# Configuration / Arguments
# ================================

ROOT_DIR="${1:-generated_markdown}"
NUM_FILES="${2:-20}"
MAX_SUBFOLDERS="${3:-5}"
MAX_HEADINGS_PER_FILE=5
MAX_LINKS_PER_FILE=4

# macOS-safe locale
export LC_ALL=C

# ================================
# Utilities
# ================================

rand_word() {
  tr -dc 'a-z' < /dev/urandom | head -c $((RANDOM % 6 + 3))
}

rand_sentence() {
  local count=$((RANDOM % 10 + 5))
  local sentence=""
  for ((i=0; i<count; i++)); do
    sentence+="$(rand_word) "
  done
  echo "$sentence." | sed 's/^./\U&/'
}

rand_paragraph() {
  local lines=$((RANDOM % 3 + 2))
  for ((i=0; i<lines; i++)); do
    rand_sentence
  done
  echo
}

# ================================
# Setup folders
# ================================

mkdir -p "$ROOT_DIR"

SUBFOLDERS=("")
for ((i=1; i<=MAX_SUBFOLDERS; i++)); do
  folder="$ROOT_DIR/$(rand_word)"
  mkdir -p "$folder"
  SUBFOLDERS+=("$folder")
done

# ================================
# Plan file locations
# ================================

FILES=()

for ((i=1; i<=NUM_FILES; i++)); do
  dir="${SUBFOLDERS[RANDOM % ${#SUBFOLDERS[@]}]}"
  name="$(rand_word)-$i.md"
  FILES+=("$dir/$name")
done

# ================================
# Generate markdown files
# ================================

for filepath in "${FILES[@]}"; do
  filename="$(basename "$filepath" .md)"
  title="$(echo "$filename" | tr 'a-z' 'A-Z')"
  headings=$((RANDOM % MAX_HEADINGS_PER_FILE + 2))
  links=$((RANDOM % MAX_LINKS_PER_FILE + 1))

  {
    # First heading = file name
    echo "# $title"
    echo

    # Additional sections
    for ((h=2; h<=headings; h++)); do
      echo "## $(rand_word | tr 'a-z' 'A-Z')"
      echo
      rand_paragraph
    done

    # Wiki links
    echo "## Related"
    echo
    for ((l=1; l<=links; l++)); do
      target="${FILES[RANDOM % ${#FILES[@]}]}"
      if [[ "$target" != "$filepath" ]]; then
        rel="${target#$ROOT_DIR/}"
        rel="${rel%.md}"

        # If file is in root, strip folder path
        if [[ "$rel" == */* ]]; then
          echo "- [[${rel}]]"
        else
          echo "- [[${rel##*/}]]"
        fi
      fi
    done
  } > "$filepath"
done

# ================================
# Done
# ================================

echo "Generated $NUM_FILES markdown files with filename-based headings and clean wiki links in '$ROOT_DIR'"
