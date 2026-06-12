#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPO="${MD_PREVIEW_GITHUB_REPO:-vorojar/md-preview}"
WORKFLOW="${MD_PREVIEW_RELEASE_WORKFLOW:-Release}"
TAG="${1:-}"

usage() {
  printf '%s\n' \
    'usage: scripts/release.sh [vX.Y.Z]' \
    '' \
    'Runs the desktop release flow in the foreground:' \
    '  1. verifies the workspace and version' \
    '  2. pushes master and the tag while skipping the background pre-push signer' \
    '  3. waits for the GitHub Release workflow' \
    '  4. signs/notarizes/staples the macOS DMG' \
    '  5. verifies Release assets, Gatekeeper, and Sparkle appcast' \
    '' \
    'Environment:' \
    '  SKIP_VERIFY=1                    skip ./scripts/verify.sh' \
    '  MD_PREVIEW_GITHUB_REPO=owner/repo override GitHub repo' \
    '  MD_PREVIEW_SIGN_SCRIPT=path       override local-first signing script' \
    '  MD_PREVIEW_SIGN_ATTEMPTS=N        signing attempts before failing (default: 2)' >&2
}

if [ "${TAG:-}" = "-h" ] || [ "${TAG:-}" = "--help" ]; then
  usage
  exit 0
fi

cargo_version() {
  awk -F\" '/^version = / { print $2; exit }' Cargo.toml
}

if [ -z "$TAG" ]; then
  TAG="v$(cargo_version)"
fi

if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: tag must look like vX.Y.Z: $TAG" >&2
  exit 2
fi

VERSION="${TAG#v}"
if [ "$(cargo_version)" != "$VERSION" ]; then
  echo "error: Cargo.toml version $(cargo_version) does not match $TAG" >&2
  exit 3
fi

require_clean_tracked_tree() {
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "error: tracked changes are present; commit them before releasing" >&2
    git status --short >&2
    exit 4
  fi
}

require_master_branch() {
  local branch
  branch="$(git branch --show-current)"
  if [ "$branch" != "master" ]; then
    echo "error: release must be run from master; current branch is ${branch:-detached}" >&2
    exit 5
  fi
}

require_tools() {
  local missing=0
  for tool in git gh curl xcrun codesign spctl; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      echo "error: required tool missing: $tool" >&2
      missing=1
    fi
  done
  if [ "$missing" -ne 0 ]; then
    exit 6
  fi
}

ensure_tag() {
  if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    local tagged
    tagged="$(git rev-list -n 1 "$TAG")"
    local head
    head="$(git rev-parse HEAD)"
    if [ "$tagged" != "$head" ]; then
      echo "error: $TAG exists but does not point at HEAD" >&2
      exit 7
    fi
    echo "[release] using existing local tag $TAG"
  else
    git tag -a "$TAG" -m "$TAG"
    echo "[release] created tag $TAG"
  fi
}

find_release_run() {
  local head
  head="$(git rev-parse HEAD)"
  for _ in $(seq 1 40); do
    local run_id
    run_id="$(
      gh run list -R "$REPO" --workflow "$WORKFLOW" --limit 20 \
        --json databaseId,event,headBranch,headSha \
        --jq ".[] | select(.event == \"push\" and .headBranch == \"$TAG\" and .headSha == \"$head\") | .databaseId" \
        | head -n 1
    )"
    if [ -n "$run_id" ]; then
      echo "$run_id"
      return 0
    fi
    sleep 3
  done
  echo "error: could not find Release workflow run for $TAG" >&2
  exit 8
}

require_release_assets() {
  local expected=(
    appcast.xml
    MD-Preview-linux-x64.tar.gz
    MD-Preview-macOS-universal.dmg
    MD-Preview-windows-x64.exe
  )
  local assets
  assets="$(gh release view "$TAG" -R "$REPO" --json assets --jq '.assets[].name')"
  for name in "${expected[@]}"; do
    if ! grep -qx "$name" <<<"$assets"; then
      echo "error: Release $TAG missing asset: $name" >&2
      echo "$assets" >&2
      exit 9
    fi
  done
}

verify_signed_outputs() {
  local dmg="target/MD-Preview-macOS-universal.dmg"
  local app="target/MD Preview.app"

  test -f "$dmg"
  test -d "$app"
  xcrun stapler validate "$dmg"
  codesign --verify --deep --strict --verbose=2 "$app"
  spctl -a -t open --context context:primary-signature "$dmg"

  curl -fsSL "https://github.com/$REPO/releases/latest/download/appcast.xml" \
    | grep -E "MD Preview $VERSION|$TAG/MD-Preview-macOS-universal\\.dmg|sparkle:edSignature"
}

require_tools
require_master_branch
require_clean_tracked_tree

if [ "${SKIP_VERIFY:-0}" != "1" ]; then
  echo "[release] running project verification"
  ./scripts/verify.sh
else
  echo "[release] SKIP_VERIFY=1; skipping ./scripts/verify.sh"
fi

ensure_tag

echo "[release] pushing master and $TAG"
MD_PREVIEW_RELEASE_FOREGROUND=1 git push origin master
MD_PREVIEW_RELEASE_FOREGROUND=1 git push origin "$TAG"

RUN_ID="$(find_release_run)"
echo "[release] watching GitHub Actions run $RUN_ID"
gh run watch "$RUN_ID" -R "$REPO" --exit-status

LOG="target/release-sign-$TAG.log"
mkdir -p target
echo "[release] signing macOS DMG in foreground; log: $LOG"
./release-sign.sh "$TAG" 2>&1 | tee "$LOG"

echo "[release] verifying published assets and signed outputs"
require_release_assets
verify_signed_outputs

echo ""
echo "DONE. $TAG released, signed, notarized, stapled, and verified."
echo "Release: https://github.com/$REPO/releases/tag/$TAG"
