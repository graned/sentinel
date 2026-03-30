#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# update_schema_patch.sh
#
# Regenerates src/schema.patch from the diff between the raw
# diesel-generated schema and the current (corrected) schema.rs.
#
# Run this after:
#   - manually correcting schema.rs (e.g. Array<Nullable<Text>> → Array<Text>)
#   - adding new migrations that introduce TEXT[] columns needing Vec<String>
#
# The generated patch is applied automatically by diesel.toml's
# patch_file setting on every `diesel migration run` / `diesel print-schema`.
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

DATABASE_URL="${DATABASE_URL:-postgresql://postgres:password@0.0.0.0:5432/sentinel_auth}"
SCHEMA_FILE="src/schema.rs"
PATCH_FILE="src/schema.patch"
RAW_SCHEMA="$(mktemp /tmp/schema_raw_XXXXXX.rs)"
PATCH_BACKUP="$(mktemp /tmp/schema_patch_bak_XXXXXX.patch)"

cleanup() {
    rm -f "$RAW_SCHEMA" "$PATCH_BACKUP"
}
trap cleanup EXIT

# ---- sanity checks ----
if ! command -v diesel >/dev/null 2>&1; then
    echo "!! diesel CLI not found in PATH" >&2
    exit 1
fi

if [[ ! -f "$SCHEMA_FILE" ]]; then
    echo "!! $SCHEMA_FILE not found — run diesel migration run first" >&2
    exit 1
fi

echo "==> Generating raw diesel schema (patch temporarily disabled)..."

# Step 1: Replace patch with an empty file so diesel print-schema applies no
#         corrections and gives the raw output. Diesel requires the file to
#         exist when patch_file is configured in diesel.toml, so we cannot
#         simply remove it.
if [[ -f "$PATCH_FILE" ]]; then
    cp "$PATCH_FILE" "$PATCH_BACKUP"
fi
: > "$PATCH_FILE"  # truncate to empty (no-op patch)

# Step 2: Capture raw diesel output.
diesel print-schema --database-url "$DATABASE_URL" > "$RAW_SCHEMA"

# Step 3: Restore the old patch (will be overwritten below, but needed in case
#         we abort early due to an error).
if [[ -s "$PATCH_BACKUP" ]]; then
    cp "$PATCH_BACKUP" "$PATCH_FILE"
fi

echo "==> Diffing raw schema against $SCHEMA_FILE..."

# Step 4: Generate new patch (raw diesel output → desired schema.rs).
#         diff exits 1 when files differ, which is expected here.
diff -U6 "$RAW_SCHEMA" "$SCHEMA_FILE" > "$PATCH_FILE" || true

if [[ ! -s "$PATCH_FILE" ]]; then
    echo "==> No differences found — schema.rs already matches diesel output."
    echo "    schema.patch cleared (nothing to patch)."
    exit 0
fi

# Step 5: Normalise the file headers to src/schema.rs so the patch is
#         reproducible regardless of the temp file path or timestamp.
sed -i "s|^--- $RAW_SCHEMA.*|--- src/schema.rs|" "$PATCH_FILE"
sed -i "s|^+++ $SCHEMA_FILE.*|+++ src/schema.rs|" "$PATCH_FILE"

echo "==> schema.patch updated. Corrections captured:"
grep "^[-+]" "$PATCH_FILE" | grep -v "^---\|^+++" | sed 's/^/    /'

echo "==> Done."
