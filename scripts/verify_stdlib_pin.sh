#!/usr/bin/env bash
# Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Resolve STDLIB_VERSION to the exact bundled stdlib commit and emit the
# release identity record (REL-20261106-1817).
#
# The pin must be an ANNOTATED tag: a tag can be force-moved, so the release
# records the peeled 40-char commit instead of trusting the tag name alone.
# This check fails when:
#   * the tag is missing or is a lightweight tag;
#   * the checkout HEAD differs from the tag's annotated commit;
#   * package.xi at that commit does not match the tag's version suffix.
#
# Output format (written to --out and --stage/lib/STDLIB_COMMIT):
#   <40-char commit SHA>
#   stdlib_version=<tag> stdlib_commit=<sha> stdlib_package_version=<version>
#
# With no arguments it verifies the repository's default `stdlib` checkout and
# prints the record. The release workflow passes --out/--stage explicitly.
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: verify_stdlib_pin.sh [--stdlib DIR] [--ref REF] [--out FILE] [--stage DIR]

  --stdlib DIR   stdlib git checkout (default: stdlib)
  --ref REF      stdlib tag (default: read STDLIB_VERSION)
  --out FILE     write the commit record to FILE
  --stage DIR    also write DIR/lib/STDLIB_COMMIT

  --print        print the record (default when no output option is given)
EOF
  exit 2
}

stdlib_dir="stdlib"
ref=""
out=""
stage=""
printed=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --stdlib)
      [ "$#" -ge 2 ] || usage
      stdlib_dir="$2"
      shift 2
      ;;
    --ref)
      [ "$#" -ge 2 ] || usage
      ref="$2"
      shift 2
      ;;
    --out)
      [ "$#" -ge 2 ] || usage
      out="$2"
      shift 2
      ;;
    --stage)
      [ "$#" -ge 2 ] || usage
      stage="$2"
      shift 2
      ;;
    --print)
      printed=1
      shift
      ;;
    -h|--help)
      usage
      ;;
    *)
      echo "verify_stdlib_pin.sh: unknown argument: $1" >&2
      usage
      ;;
  esac
done

fail() {
  echo "::error::stdlib pin: $*" >&2
  exit 1
}

[ -d "$stdlib_dir" ] || fail "stdlib checkout not found: $stdlib_dir"
git -C "$stdlib_dir" rev-parse --git-dir >/dev/null 2>&1 \
  || fail "$stdlib_dir is not a git checkout"

if [ -z "$ref" ]; then
  [ -f STDLIB_VERSION ] || fail "STDLIB_VERSION not found and --ref was not given"
  ref="$(tr -d '\r\n ' < STDLIB_VERSION)"
fi
[ -n "$ref" ] || fail "STDLIB_VERSION is empty"

case "$ref" in
  *[!A-Za-z0-9._/-]*)
    fail "unsafe ref name: $ref"
    ;;
esac
case "$ref" in
  stdlib-v*)
    tag_version="${ref#stdlib-v}"
    ;;
  *)
    fail "STDLIB_VERSION must name a stdlib-v* release tag (got: $ref)"
    ;;
esac

tag_ref="refs/tags/$ref"
tag_meta="$(git -C "$stdlib_dir" for-each-ref \
  --format='%(objecttype) %(objectname) %(*objecttype) %(*objectname)' \
  "$tag_ref")"
[ -n "$tag_meta" ] \
  || fail "tag $ref does not exist in $stdlib_dir (fetch tags before running)"

read -r tag_type _ target_type target_commit <<<"$tag_meta"

[ "$tag_type" = "tag" ] \
  || fail "$ref is a $tag_type tag, not an annotated tag; re-tag it as annotated (do not move the release tag)"
[ "$target_type" = "commit" ] \
  || fail "$ref does not directly point at a commit (target type: $target_type)"
[[ "$target_commit" =~ ^[0-9a-f]{40}$ ]] \
  || fail "$ref resolved to a non-40-char commit: $target_commit"

head_commit="$(git -C "$stdlib_dir" rev-parse --verify 'HEAD^{commit}')"
[ "$head_commit" = "$target_commit" ] \
  || fail "moved/mismatched tag: checkout HEAD $head_commit != $ref annotated commit $target_commit"

if ! package_xi="$(git -C "$stdlib_dir" show 'HEAD:package.xi' 2>/dev/null)"; then
  fail "cannot read package.xi at $target_commit"
fi
package_version=""
while IFS= read -r package_line; do
  if [[ "$package_line" =~ ^[[:space:]]*version:[[:space:]]*\"([^\"]*)\" ]]; then
    package_version="${BASH_REMATCH[1]}"
    break
  fi
done <<<"$package_xi"
[ -n "$package_version" ] \
  || fail "cannot parse the version field from package.xi at $target_commit"
[ "$package_version" = "$tag_version" ] \
  || fail "tag $ref names version $tag_version but package.xi at $target_commit says $package_version (moved or wrong tag)"

record="$target_commit
stdlib_version=$ref stdlib_commit=$target_commit stdlib_package_version=$package_version"

write_record() {
  local path="$1"
  mkdir -p "$(dirname "$path")"
  printf '%s\n' "$record" > "$path"
}

if [ -n "$out" ]; then
  write_record "$out"
fi
if [ -n "$stage" ]; then
  write_record "$stage/lib/STDLIB_COMMIT"
fi
if [ "$printed" -eq 1 ] || { [ -z "$out" ] && [ -z "$stage" ]; }; then
  printf '%s\n' "$record"
fi
