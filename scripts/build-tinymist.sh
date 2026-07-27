#!/usr/bin/env bash
#
# Build and stage tinymist binaries for the server's LSP config (`lsp.versions`).
#
# tinymist can only be obtained as a subprocess binary: the crate is not usable
# as a library dependency (it builds against a patched Typst fork wired via a
# workspace `[patch.crates-io]`), and crates.io ships no binary. So each Typst
# version we support maps to a `tinymist` binary built from a pinned git tag and
# staged on disk; the server's `lsp.versions[].binary` points at the result.
#
# Usage:
#   scripts/build-tinymist.sh <out_dir> <typst_version>=<tinymist_tag> [...]
#
# Example (2-3 versions, matching `lsp.default_version`/`versions` in config):
#   scripts/build-tinymist.sh /opt/tinymist \
#     0.12.0=v0.12.21 \
#     0.13.1=v0.13.16
#
# Produces:
#   <out_dir>/<typst_version>/tinymist
#
# Each build is heavy (tinymist + its patched typst). Run it in the image build
# (once, cached) or an ops box, not per-deploy. The binaries are self-contained
# and offline at runtime.
set -euo pipefail

REPO="https://github.com/Myriad-Dreamin/tinymist.git"

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <out_dir> <typst_version>=<tinymist_tag> [...]" >&2
  exit 2
fi

OUT_DIR="$1"
shift

command -v cargo >/dev/null || { echo "cargo not found; a Rust toolchain is required" >&2; exit 1; }
command -v git >/dev/null || { echo "git not found" >&2; exit 1; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

for pair in "$@"; do
  typst_version="${pair%%=*}"
  tinymist_tag="${pair#*=}"
  if [[ "$typst_version" == "$pair" || -z "$tinymist_tag" ]]; then
    echo "bad argument '$pair'; expected <typst_version>=<tinymist_tag>" >&2
    exit 2
  fi

  echo ">> tinymist $tinymist_tag  (Typst $typst_version)"
  src="$WORK/$tinymist_tag"
  # Shallow clone of just the tag, to keep the checkout small.
  git clone --depth 1 --branch "$tinymist_tag" "$REPO" "$src"

  # The server binary crate is `tinymist` (a.k.a. the CLI); build it release.
  ( cd "$src" && cargo build --release --locked -p tinymist )

  dest="$OUT_DIR/$typst_version"
  mkdir -p "$dest"
  install -m 0755 "$src/target/release/tinymist" "$dest/tinymist"

  # Sanity: the binary reports the Typst version it compiles. Warn (don't fail)
  # if it doesn't match the label we staged it under.
  reported="$("$dest/tinymist" --version 2>/dev/null || true)"
  echo "   staged $dest/tinymist"
  echo "   reports: ${reported:-<no --version output>}"
  case "$reported" in
    *"$typst_version"*) : ;;
    *) echo "   WARNING: '--version' does not mention $typst_version; verify the tag→version mapping" >&2 ;;
  esac
done

echo "done: binaries under $OUT_DIR/<typst_version>/tinymist"
