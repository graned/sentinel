#!/usr/bin/env bash
# scripts/ci.sh — Sentinel CI helper
#
# Encapsulates all build/lint/test logic so CI YAML stays thin and every
# step is runnable locally by exporting the required env vars.
#
# Commands:
#   lint   <core|ui>
#   build  <core|ui>  --sha <sha> --event <pr|commit>
#                     [--pr <n>] [--changed <bool>] [--build-arg KEY=VAL]
#   test                              reads: SENTINEL_CORE_IMAGE,
#                                            SENTINEL_MIGRATE_IMAGE
#   promote <core|ui> --pr-sha <sha> --merge-sha <sha>
#   cleanup-pr        --pr <n>        reads: GH_TOKEN
#
# Required env vars for build/promote:
#   TANGO_REGISTRY_HOST, TANGO_REGISTRY_NAMESPACE,
#   TANGO_REGISTRY_USER, TANGO_REGISTRY_PASSWORD

set -euo pipefail

COMMAND="${1:-}"
shift || true

# ── Helpers ──────────────────────────────────────────────────────────────────

_setup_project() {
  case "$1" in
    core) export TANGO_PACKAGE="sentinel-core"; _PKG_IMAGE="sentinel-core" ;;
    ui)   export TANGO_PACKAGE="sentinel-ui";   _PKG_IMAGE="sentinel-ui"   ;;
    *)    echo "Unknown project: $1 (expected: core|ui)" >&2; exit 1 ;;
  esac
  _IMAGE_REF="${TANGO_REGISTRY_HOST:-ghcr.io}/${TANGO_REGISTRY_NAMESPACE:-}/${_PKG_IMAGE}"
}

_image_exists() {
  docker buildx imagetools inspect "$1" >/dev/null 2>&1
}

# ── lint ─────────────────────────────────────────────────────────────────────

cmd_lint() {
  local project="${1:-}"
  case "$project" in
    core)
      cargo fmt --all -- --check
      cargo clippy --all-targets --all-features -- -D warnings
      ;;
    ui)
      # Build each package in dependency order: sdk → auth-react → ui.
      # dist/ from each step is consumed by the next via file: dependencies.
      (cd packages/sentinel-auth-sdk && npm ci && npm run typecheck && npm run build)
      (cd packages/sentinel-auth-react && npm install --legacy-peer-deps && npm run typecheck && npm run build)
      export VITE_API_URL="${VITE_API_URL:-http://localhost:3000}"
      (cd apps/sentinel-ui && npm install --legacy-peer-deps && npx tsc --noEmit && npm run build)
      ;;
    *)
      echo "Usage: ci.sh lint <core|ui>" >&2; exit 1 ;;
  esac
}

# ── build ─────────────────────────────────────────────────────────────────────

cmd_build() {
  local project="${1:-}"; shift || true
  _setup_project "$project"

  local sha="" event="" pr="" changed="true"
  local -a build_args=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --sha)       sha="$2";                       shift 2 ;;
      --event)     event="$2";                     shift 2 ;;
      --pr)        pr="$2";                        shift 2 ;;
      --changed)   changed="$2";                   shift 2 ;;
      --build-arg) build_args+=(--build-arg "$2"); shift 2 ;;
      *) echo "Unknown flag: $1" >&2; exit 1 ;;
    esac
  done

  case "$event" in
    pr)
      if [[ "$changed" == "true" ]]; then
        echo "Building ${TANGO_PACKAGE} for PR #${pr} (files changed)"
        tango dance \
          --package "$TANGO_PACKAGE" \
          --event pull-request \
          --sha "$sha" \
          --pr "$pr" \
          "${build_args[@]}"
      elif _image_exists "${_IMAGE_REF}:latest"; then
        echo "No ${TANGO_PACKAGE} changes — retagging ${_IMAGE_REF}:latest → pr-${pr}"
        tango promote \
          --package "$TANGO_PACKAGE" \
          --source "${_IMAGE_REF}:latest" \
          --target-tag "pr-${pr}"
      else
        echo "No latest tag found — falling back to full build for ${TANGO_PACKAGE}"
        tango dance \
          --package "$TANGO_PACKAGE" \
          --event pull-request \
          --sha "$sha" \
          --pr "$pr" \
          "${build_args[@]}"
      fi
      ;;
    commit)
      echo "Building ${TANGO_PACKAGE} for commit ${sha}"
      tango dance \
        --package "$TANGO_PACKAGE" \
        --event commit \
        --sha "$sha" \
        "${build_args[@]}"
      tango promote \
        --package "$TANGO_PACKAGE" \
        --sha "$sha" \
        --target-tag latest
      ;;
    *)
      echo "Unknown event: $event (expected: pr|commit)" >&2; exit 1 ;;
  esac
}

# ── test ─────────────────────────────────────────────────────────────────────

cmd_test() {
  : "${SENTINEL_CORE_IMAGE:?SENTINEL_CORE_IMAGE must be set}"
  : "${SENTINEL_MIGRATE_IMAGE:?SENTINEL_MIGRATE_IMAGE must be set}"

  docker pull "$SENTINEL_MIGRATE_IMAGE"
  docker pull "$SENTINEL_CORE_IMAGE"

  trap 'docker compose -f docker-compose.ci.yml down --volumes' EXIT

  docker compose -f docker-compose.ci.yml up -d sentinel-core || {
    echo "=== docker compose up failed — dumping logs ==="
    docker compose -f docker-compose.ci.yml logs
    exit 1
  }

  echo "Waiting for sentinel-core to become healthy..."
  for i in $(seq 1 36); do
    CID=$(docker compose -f docker-compose.ci.yml ps -q sentinel-core)
    if [ -z "$CID" ]; then
      echo "sentinel-core container not found — logs:"
      docker compose -f docker-compose.ci.yml logs
      exit 1
    fi
    HEALTH=$(docker inspect --format='{{.State.Health.Status}}' "$CID" 2>/dev/null || echo "unknown")
    STATE=$(docker inspect  --format='{{.State.Status}}'        "$CID" 2>/dev/null || echo "unknown")
    echo "  [$i/36] state=$STATE health=$HEALTH"
    if [ "$HEALTH" = "healthy" ]; then
      echo "sentinel-core is healthy"; break
    fi
    if [ "$STATE" = "exited" ] || [ "$STATE" = "dead" ]; then
      echo "sentinel-core exited unexpectedly — logs:"
      docker compose -f docker-compose.ci.yml logs sentinel-core
      exit 1
    fi
    if [ "$i" -eq 36 ]; then
      echo "Timed out waiting for sentinel-core — logs:"
      docker compose -f docker-compose.ci.yml logs sentinel-core
      exit 1
    fi
    sleep 5
  done

  docker compose -f docker-compose.ci.yml run --rm test-runner
}

# ── promote ──────────────────────────────────────────────────────────────────

cmd_promote() {
  local project="${1:-}"; shift || true
  _setup_project "$project"

  local pr_sha="" merge_sha=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --pr-sha)    pr_sha="$2";    shift 2 ;;
      --merge-sha) merge_sha="$2"; shift 2 ;;
      *) echo "Unknown flag: $1" >&2; exit 1 ;;
    esac
  done

  local short_pr="${pr_sha:0:12}"
  local short_merge="${merge_sha:0:12}"

  if _image_exists "${_IMAGE_REF}:sha-${short_pr}"; then
    tango promote \
      --package "$TANGO_PACKAGE" \
      --sha "$pr_sha" \
      --target-tag "sha-${short_merge}" \
      --target-tag latest
  else
    echo "${_IMAGE_REF}:sha-${short_pr} not found — PR had no changes for ${TANGO_PACKAGE}, skipping promote (latest already current)"
  fi
}

# ── cleanup-pr ───────────────────────────────────────────────────────────────

cmd_cleanup_pr() {
  local pr=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --pr) pr="$2"; shift 2 ;;
      *) echo "Unknown flag: $1" >&2; exit 1 ;;
    esac
  done
  : "${GH_TOKEN:?GH_TOKEN must be set}"
  : "${pr:?--pr is required}"

  for pkg in sentinel-core sentinel-ui; do
    VERSION_ID=$(gh api "/user/packages/container/${pkg}/versions" \
      --paginate \
      --jq ".[] | select(.metadata.container.tags[] == \"pr-${pr}\") | .id" \
      2>/dev/null | head -1)
    if [ -n "$VERSION_ID" ]; then
      gh api --method DELETE "/user/packages/container/${pkg}/versions/${VERSION_ID}"
      echo "Deleted ${pkg}:pr-${pr} (version ${VERSION_ID})"
    else
      echo "Tag pr-${pr} not found in ${pkg} — skipping delete"
    fi
  done
}

# ── Dispatch ─────────────────────────────────────────────────────────────────

case "$COMMAND" in
  lint)       cmd_lint       "$@" ;;
  build)      cmd_build      "$@" ;;
  test)       cmd_test              ;;
  promote)    cmd_promote    "$@" ;;
  cleanup-pr) cmd_cleanup_pr "$@" ;;
  *)
    cat >&2 <<'EOF'
Usage: ci.sh <command> [args]

Commands:
  lint   <core|ui>
  build  <core|ui>  --sha <sha> --event <pr|commit>
                    [--pr <n>] [--changed <bool>] [--build-arg KEY=VAL]
  test                              reads: SENTINEL_CORE_IMAGE, SENTINEL_MIGRATE_IMAGE
  promote <core|ui> --pr-sha <sha> --merge-sha <sha>
  cleanup-pr        --pr <n>        reads: GH_TOKEN

Required env vars for build/promote:
  TANGO_REGISTRY_HOST, TANGO_REGISTRY_NAMESPACE,
  TANGO_REGISTRY_USER, TANGO_REGISTRY_PASSWORD
EOF
    exit 1
    ;;
esac
