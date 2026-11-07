#!/usr/bin/env bash
# Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Local/dry-run proof for REL-20261106-1817: resolve STDLIB_VERSION, stage
# lib/STDLIB_COMMIT, build a throwaway archive, and verify the archive carries
# the exact commit. It never tags, commits, pushes, or touches the release
# workflow.
#
# Usage:
#   scripts/dry_run_stdlib_commit.sh [--stdlib DIR] [--ref TAG] [--remote URL]
#
# --stdlib DIR checks an existing clone (or worktree) at the pin. Without it
# the script shallow-clones STDLIB_VERSION from --remote (default
# https://github.com/xiom-lang/stdlib.git) into a temporary directory.
# Everything created by this script lives under mktemp and is removed on exit.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
stdlib_source="${STDLIB_DIR:-}"
remote="${STDLIB_REMOTE:-https://github.com/xiom-lang/stdlib.git}"
ref="${STDLIB_REF:-}"

usage() {
  cat >&2 <<'EOF'
usage: dry_run_stdlib_commit.sh [--stdlib DIR] [--ref TAG] [--remote URL]

  --stdlib DIR  existing stdlib clone/worktree at the pin (no clone)
  --ref TAG     stdlib tag (default: STDLIB_VERSION)
  --remote URL  stdlib remote for the temporary shallow clone
EOF
  exit 2
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --stdlib)
      [ "$#" -ge 2 ] || usage
      stdlib_source="$2"
      shift 2
      ;;
    --ref)
      [ "$#" -ge 2 ] || usage
      ref="$2"
      shift 2
      ;;
    --remote)
      [ "$#" -ge 2 ] || usage
      remote="$2"
      shift 2
      ;;
    -h|--help)
      usage
      ;;
    *)
      echo "dry_run_stdlib_commit.sh: unknown argument: $1" >&2
      usage
      ;;
  esac
done

if [ -z "$ref" ]; then
  ref="$(tr -d '\r\n ' < "$repo_root/STDLIB_VERSION")"
fi

work="$(mktemp -d)"
cleanup() {
  rm -rf "$work"
}
trap cleanup EXIT
trap 'exit 130' INT TERM

if [ -n "$stdlib_source" ]; then
  [ -d "$stdlib_source" ] \
    || { echo "dry run: stdlib checkout not found: $stdlib_source" >&2; exit 1; }
  stdlib_dir="$stdlib_source"
else
  git clone --quiet --depth 1 --branch "$ref" "$remote" "$work/stdlib"
  stdlib_dir="$work/stdlib"
fi

bash "$repo_root/scripts/verify_stdlib_pin.sh" \
  --stdlib "$stdlib_dir" \
  --ref "$ref" \
  --out "$work/STDLIB_COMMIT" \
  --stage "$work/stage"

sha="$(head -n 1 "$work/STDLIB_COMMIT")"
staged_sha="$(head -n 1 "$work/stage/lib/STDLIB_COMMIT")"
[ "$staged_sha" = "$sha" ] \
  || { echo "dry run: staged lib/STDLIB_COMMIT ($staged_sha) != $sha" >&2; exit 1; }

tar -czf "$work/asset.tar.gz" -C "$work/stage" .
if ! tar -tzf "$work/asset.tar.gz" | sed 's|^\./||' | grep -qx 'lib/STDLIB_COMMIT'; then
  echo "dry run: simulated asset is missing lib/STDLIB_COMMIT" >&2
  exit 1
fi
archived_sha="$(tar -xzOf "$work/asset.tar.gz" ./lib/STDLIB_COMMIT | head -n 1)"
[ "$archived_sha" = "$sha" ] \
  || { echo "dry run: simulated asset commit ($archived_sha) != $sha" >&2; exit 1; }

record="$(sed -n '2p' "$work/STDLIB_COMMIT")"
echo "dry run OK: $record"
echo "dry run: staged lib/STDLIB_COMMIT and verified $work/asset.tar.gz (temporary)"
