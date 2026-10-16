#!/usr/bin/env bash
set -euo pipefail

# Include embedded themes, icons and syntax definitions as well as Rust code.
# Git tree entries make this independent of checkout timestamps and docs edits.
git ls-tree -r HEAD -- Cargo.toml Cargo.lock src .cargo build.rs rust-toolchain rust-toolchain.toml |
  sha256sum | cut -d ' ' -f 1
