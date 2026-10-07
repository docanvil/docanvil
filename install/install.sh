#!/bin/sh
# DocAnvil installer for macOS and Linux.
#
#   curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh
#   curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh -s -- --version 1.2.0
#
# Options (environment variables in brackets):
#   --version X.Y.Z     install a specific version (default: latest)   [DOCANVIL_VERSION]
#   --install-dir DIR   where to put the binary (default: ~/.local/bin) [DOCANVIL_INSTALL_DIR]
#   --force             reinstall even if that version is already installed
#   --quiet             only print errors

set -eu

REPO_URL="${DOCANVIL_REPO_URL:-https://github.com/docanvil/docanvil}"
API_URL="${DOCANVIL_API_URL:-https://api.github.com/repos/docanvil/docanvil}"

usage() {
  cat <<'EOF'
DocAnvil installer for macOS and Linux.

  curl -fsSL https://github.com/docanvil/docanvil/releases/latest/download/install.sh | sh

Options (environment variables in brackets):
  --version X.Y.Z     install a specific version (default: latest)   [DOCANVIL_VERSION]
  --install-dir DIR   where to put the binary (default: ~/.local/bin) [DOCANVIL_INSTALL_DIR]
  --force             reinstall even if that version is already installed
  --quiet             only print errors
EOF
}

main() {
  VERSION="${DOCANVIL_VERSION:-}"
  INSTALL_DIR="${DOCANVIL_INSTALL_DIR:-$HOME/.local/bin}"
  FORCE=0
  QUIET=0

  while [ $# -gt 0 ]; do
    case "$1" in
      --version) [ $# -ge 2 ] || err "--version needs a value"; VERSION="$2"; shift 2 ;;
      --version=*) VERSION="${1#*=}"; shift ;;
      --install-dir) [ $# -ge 2 ] || err "--install-dir needs a value"; INSTALL_DIR="$2"; shift 2 ;;
      --install-dir=*) INSTALL_DIR="${1#*=}"; shift ;;
      --force) FORCE=1; shift ;;
      -q|--quiet) QUIET=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) err "unknown option: $1 (try --help)" ;;
    esac
  done
  VERSION="${VERSION#v}"

  need_tools
  detect_target

  if [ -z "$VERSION" ]; then
    loc=$(latest_location "$REPO_URL/releases/latest") || loc=""
    VERSION="${loc##*/}"
    VERSION="${VERSION#v}"
  fi
  case "$VERSION" in
    [0-9]*.[0-9]*.[0-9]*) ;;
    *) err "couldn't work out which DocAnvil version to install (got '${VERSION:-nothing}'). Pass --version X.Y.Z to choose one." ;;
  esac

  BIN="$INSTALL_DIR/docanvil"
  if [ "$FORCE" = 0 ]; then
    existing=""
    if [ -x "$BIN" ]; then
      existing="$BIN"
    elif command -v docanvil >/dev/null 2>&1; then
      existing=$(command -v docanvil)
    fi
    if [ -n "$existing" ]; then
      have=$("$existing" --version 2>/dev/null | awk '{print $2}') || have=""
      if [ "$have" = "$VERSION" ]; then
        say "✅ docanvil v$VERSION is already installed at $existing"
        exit 0
      fi
    fi
  fi

  ASSET="docanvil-v$VERSION-$TARGET.tar.gz"
  BASE="$REPO_URL/releases/download/v$VERSION"
  TMP=$(mktemp -d 2>/dev/null || mktemp -d -t docanvil)
  STAGED=""
  # Clean up on any exit. A trap replaces a signal's default action, so INT and
  # TERM have to exit explicitly (which then runs the EXIT trap) or the script
  # would carry on after Ctrl-C.
  trap cleanup EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM

  say "⬇️  Downloading docanvil v$VERSION ($TARGET)"
  fetch "$BASE/$ASSET" "$TMP/$ASSET" \
    || err "download failed: $BASE/$ASSET (does v$VERSION exist? see $REPO_URL/releases)"

  expected=$(expected_sha "$ASSET")
  [ -n "$expected" ] || err "couldn't find a published checksum for $ASSET; refusing to install an unverified binary. (Releases up to v1.1.3 are checked through GitHub's API, which may be rate-limiting you: set GITHUB_TOKEN and try again.)"
  actual=$(sha256 "$TMP/$ASSET")
  [ "$actual" = "$expected" ] || err "checksum mismatch for $ASSET (expected $expected, got $actual)"

  tar -xzf "$TMP/$ASSET" -C "$TMP"
  [ -f "$TMP/docanvil" ] || err "the archive didn't contain a docanvil binary"

  mkdir -p "$INSTALL_DIR" || err "couldn't create $INSTALL_DIR (try --install-dir)"
  STAGED="$INSTALL_DIR/.docanvil.tmp.$$"
  cp "$TMP/docanvil" "$STAGED" 2>/dev/null \
    || err "couldn't write to $INSTALL_DIR (try --install-dir, or run with sudo)"
  chmod 755 "$STAGED"
  # Make sure it actually runs here before replacing anything.
  if ! "$STAGED" --version >/dev/null 2>&1; then
    err "the downloaded docanvil binary won't run on this system ($TARGET). Install from source with: cargo install docanvil"
  fi
  mv -f "$STAGED" "$BIN"
  STAGED=""

  say "✅ Installed docanvil v$VERSION to $BIN"
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      say ""
      say "$INSTALL_DIR isn't on your PATH yet. Add this to your shell profile:"
      say "  export PATH=\"$INSTALL_DIR:\$PATH\""
      ;;
  esac
}

cleanup() {
  rm -rf "$TMP"
  # A half-written or unverified binary must never be left in the install dir.
  [ -z "$STAGED" ] || rm -f "$STAGED"
}

say() { [ "$QUIET" = 1 ] || printf '%s\n' "$*"; }
err() { printf 'error: %s\n' "$*" >&2; exit 1; }

need_tools() {
  if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsL "$1" -o "$2"; }
    api_fetch() {
      if [ -n "${GITHUB_TOKEN:-}" ]; then
        curl -fsL -H "Authorization: Bearer $GITHUB_TOKEN" "$1" -o "$2"
      else
        fetch "$1" "$2"
      fi
    }
    latest_location() { curl -fsSI "$1" | tr -d '\r' | awk 'tolower($1) == "location:" { print $2 }' | head -n 1; }
  elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -q -O "$2" "$1"; }
    api_fetch() {
      if [ -n "${GITHUB_TOKEN:-}" ]; then
        wget -q --header="Authorization: Bearer $GITHUB_TOKEN" -O "$2" "$1"
      else
        fetch "$1" "$2"
      fi
    }
    # wget follows the redirect; the first Location header is the tag URL.
    latest_location() { wget -S --spider "$1" 2>&1 | tr -d '\r' | awk 'tolower($1) == "location:" { print $2 }' | head -n 1; }
  else
    err "DocAnvil's installer needs curl or wget"
  fi
  command -v tar >/dev/null 2>&1 || err "DocAnvil's installer needs tar"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | awk '{ print $1 }'; }
  elif command -v shasum >/dev/null 2>&1; then
    sha256() { shasum -a 256 "$1" | awk '{ print $1 }'; }
  else
    err "DocAnvil's installer needs sha256sum or shasum to verify the download"
  fi
}

detect_target() {
  os=$(uname -s)
  arch=$(uname -m)
  case "$os/$arch" in
    Linux/x86_64 | Linux/amd64) TARGET=x86_64-unknown-linux-musl ;;
    Linux/aarch64 | Linux/arm64) TARGET=aarch64-unknown-linux-musl ;;
    Darwin/arm64 | Darwin/aarch64) TARGET=aarch64-apple-darwin ;;
    Darwin/x86_64) TARGET=x86_64-apple-darwin ;;
    *) err "no prebuilt DocAnvil for $os/$arch. Install from source with: cargo install docanvil" ;;
  esac
}

# Prints the expected SHA-256 for $1, or nothing.
expected_sha() {
  if fetch "$BASE/SHA256SUMS" "$TMP/SHA256SUMS" 2>/dev/null; then
    awk -v a="$1" '{ n = $2; sub(/^\*/, "", n) } n == a { print tolower($1); exit }' "$TMP/SHA256SUMS"
  elif api_fetch "$API_URL/releases/tags/v$VERSION" "$TMP/release.json" 2>/dev/null; then
    # Releases before SHA256SUMS existed: use GitHub's per-asset digest.
    # The API allows 60 anonymous requests an hour; GITHUB_TOKEN raises that.
    # Asset "name" always precedes its "digest" in the API response.
    grep -oE '"(name|digest)": *"[^"]*"' "$TMP/release.json" \
      | awk -v a="$1" '
          /^"name"/ { sub(/^"name": *"/, ""); sub(/"$/, ""); n = $0 }
          /^"digest"/ && n == a { sub(/^"digest": *"sha256:/, ""); sub(/"$/, ""); print tolower($0); exit }'
  fi
}

main "$@"
