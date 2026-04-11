#!/usr/bin/env bash
# release.sh — create and push a semver release tag for sentinel.
#
# The tag push triggers .github/workflows/release.yml which promotes the
# already-built Docker images (sentinel-core + sentinel-ui) to the new
# release version without rebuilding.
#
# Usage:
#   ./scripts/release.sh major          v1.0.0  → v2.0.0
#   ./scripts/release.sh minor          v1.0.0  → v1.1.0
#   ./scripts/release.sh patch          v1.0.0  → v1.0.1
#   ./scripts/release.sh alpha          v1.0.0  → v1.0.1-alpha.1
#   ./scripts/release.sh alpha          v1.0.1-alpha.1 → v1.0.1-alpha.2
#   ./scripts/release.sh beta           v1.0.1-alpha.2 → v1.0.1-beta.1
#   ./scripts/release.sh beta           v1.0.1-beta.1  → v1.0.1-beta.2
#   ./scripts/release.sh patch          v1.0.1-beta.2  → v1.0.2  (stable release)
#
# Flags:
#   --dry-run   Print what would happen without creating or pushing the tag.

set -euo pipefail

# ── Helpers ──────────────────────────────────────────────────────────────────

usage() {
  sed -n '/^# Usage:/,/^$/p' "$0" | sed 's/^# \{0,2\}//'
  exit 1
}

die() { echo "error: $*" >&2; exit 1; }

# Fetch all tags from origin and return the highest semver tag, or empty.
latest_tag() {
  git fetch --tags --quiet 2>/dev/null || true
  # sort -V handles semver ordering including pre-release suffixes.
  git tag -l 'v[0-9]*' | sort -V | tail -1
}

# Parse a vX.Y.Z[-type.N] tag into global vars.
# Sets: MAJOR MINOR PATCH PRE_TYPE PRE_NUM (pre-release fields empty for stable)
parse_tag() {
  local raw="${1#v}"
  if [[ "$raw" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)-([a-zA-Z]+)\.([0-9]+)$ ]]; then
    MAJOR="${BASH_REMATCH[1]}"
    MINOR="${BASH_REMATCH[2]}"
    PATCH="${BASH_REMATCH[3]}"
    PRE_TYPE="${BASH_REMATCH[4]}"
    PRE_NUM="${BASH_REMATCH[5]}"
  elif [[ "$raw" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    MAJOR="${BASH_REMATCH[1]}"
    MINOR="${BASH_REMATCH[2]}"
    PATCH="${BASH_REMATCH[3]}"
    PRE_TYPE=""
    PRE_NUM=""
  else
    die "cannot parse version '${1}'"
  fi
}

# ── Argument parsing ──────────────────────────────────────────────────────────

DRY_RUN=false
TYPE=""

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    major|minor|patch|alpha|beta) TYPE="$arg" ;;
    *) die "unknown argument '${arg}'" ;;
  esac
done

[[ -z "$TYPE" ]] && usage

# ── Determine current version ─────────────────────────────────────────────────

LATEST=$(latest_tag)
if [[ -z "$LATEST" ]]; then
  LATEST="v0.0.0"
  echo "No existing tags found — starting from v0.0.0"
else
  echo "Latest tag:  $LATEST"
fi

parse_tag "$LATEST"

# ── Compute next version ──────────────────────────────────────────────────────

case "$TYPE" in
  major)
    NEXT="v$((MAJOR + 1)).0.0"
    ;;
  minor)
    NEXT="v${MAJOR}.$((MINOR + 1)).0"
    ;;
  patch)
    # If the latest is a pre-release, 'patch' cuts a stable release from it.
    # If latest is already stable, bump the patch number.
    if [[ -n "$PRE_TYPE" ]]; then
      NEXT="v${MAJOR}.${MINOR}.${PATCH}"
    else
      NEXT="v${MAJOR}.${MINOR}.$((PATCH + 1))"
    fi
    ;;
  alpha)
    if [[ "$PRE_TYPE" == "alpha" ]]; then
      # Already in alpha series — increment number.
      NEXT="v${MAJOR}.${MINOR}.${PATCH}-alpha.$((PRE_NUM + 1))"
    elif [[ "$PRE_TYPE" == "beta" ]]; then
      # Can't go backwards; start a fresh alpha on the next patch.
      NEXT="v${MAJOR}.${MINOR}.$((PATCH + 1))-alpha.1"
    else
      # Stable → start alpha on next patch.
      NEXT="v${MAJOR}.${MINOR}.$((PATCH + 1))-alpha.1"
    fi
    ;;
  beta)
    if [[ "$PRE_TYPE" == "alpha" ]]; then
      # Promote alpha series to beta — same base version, reset pre-release num.
      NEXT="v${MAJOR}.${MINOR}.${PATCH}-beta.1"
    elif [[ "$PRE_TYPE" == "beta" ]]; then
      # Already in beta series — increment number.
      NEXT="v${MAJOR}.${MINOR}.${PATCH}-beta.$((PRE_NUM + 1))"
    else
      # Stable → start beta on next patch.
      NEXT="v${MAJOR}.${MINOR}.$((PATCH + 1))-beta.1"
    fi
    ;;
esac

# ── Dry run ───────────────────────────────────────────────────────────────────

echo "Next version: $NEXT"

if $DRY_RUN; then
  echo ""
  echo "(dry run — no tag created)"
  exit 0
fi

# ── Confirm & tag ─────────────────────────────────────────────────────────────

echo ""
read -rp "Create and push tag $NEXT? [y/N] " confirm
[[ "$confirm" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 0; }

git tag "$NEXT"
git push origin "$NEXT"

echo ""
echo "Tag $NEXT pushed."
echo "GitHub Actions will promote sentinel-core and sentinel-ui images to $NEXT."
