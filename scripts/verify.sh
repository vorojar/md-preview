#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "[agent-verify] project: $ROOT"

ran=0

AGENT_WORKFLOW="/Users/longjiewu/.agent-workflow/scripts/agent_workflow.py"
if [ -x "$AGENT_WORKFLOW" ]; then
  echo "[agent-verify] guard"
  python3 "$AGENT_WORKFLOW" guard "$ROOT"
fi

has_npm_script() {
  local script="$1"
  command -v node >/dev/null 2>&1 || return 1
  node -e "const p=require('./package.json'); process.exit(p.scripts && p.scripts[process.argv[1]] ? 0 : 1)" "$script" 2>/dev/null
}

if [ -f package.json ]; then
  if command -v npm >/dev/null 2>&1; then
    if has_npm_script test; then
      echo "[agent-verify] npm test"
      npm test
      ran=1
    fi
    if has_npm_script lint; then
      echo "[agent-verify] npm run lint"
      npm run lint
      ran=1
    fi
    if has_npm_script build; then
      echo "[agent-verify] npm run build"
      npm run build
      ran=1
    fi
  fi
fi

if [ -f pyproject.toml ] || [ -f pytest.ini ] || [ -d tests ]; then
  if command -v pytest >/dev/null 2>&1; then
    echo "[agent-verify] pytest"
    pytest
    ran=1
  fi
fi

if [ -f Makefile ] && grep -qE '^verify:' Makefile; then
  echo "[agent-verify] make verify"
  make verify
  ran=1
fi

if [ -f Cargo.toml ]; then
  if command -v cargo >/dev/null 2>&1; then
    echo "[agent-verify] cargo test"
    cargo test
    ran=1
  fi
fi

if [ -x scripts/verify-sparkle-update.sh ]; then
  echo "[agent-verify] Sparkle update"
  scripts/verify-sparkle-update.sh
  ran=1
fi

if [ -x scripts/verify-windows-update.sh ]; then
  echo "[agent-verify] WinSparkle update"
  scripts/verify-windows-update.sh
  ran=1
fi

if [ -f mobile/ios/project.yml ]; then
  if command -v xcodegen >/dev/null 2>&1 && command -v xcodebuild >/dev/null 2>&1; then
    echo "[agent-verify] iOS xcodegen"
    (
      cd mobile/ios
      xcodegen generate
      if xcodebuild -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -showdestinations 2>&1 | grep -q "not installed"; then
        echo "[agent-verify] skip iOS build: Xcode reports the iOS platform is not installed"
      else
        xcodebuild -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build
      fi
    )
    if command -v xcrun >/dev/null 2>&1; then
      echo "[agent-verify] iOS Swift parse"
      xcrun --sdk iphoneos swiftc -parse \
        mobile/ios/MDPreviewMobile/AppDelegate.swift \
        mobile/ios/MDPreviewMobile/PreviewViewController.swift
    fi
    ran=1
  else
    echo "[agent-verify] skip iOS build: xcodegen or xcodebuild missing"
  fi
fi

if [ -f mobile/android/settings.gradle ]; then
  if command -v gradle >/dev/null 2>&1; then
    echo "[agent-verify] Android assembleDebug"
    (
      cd mobile/android
      gradle :app:assembleDebug
    )
    ran=1
  else
    echo "[agent-verify] skip Android build: gradle missing"
  fi
fi

if [ -f mobile/scripts/verify-mobile-renderer.mjs ]; then
  BUNDLED_NODE="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
  BUNDLED_NODE_MODULES="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules"
  if [ -x "$BUNDLED_NODE" ] && [ -d "$BUNDLED_NODE_MODULES" ]; then
    echo "[agent-verify] mobile renderer"
    NODE_PATH="$BUNDLED_NODE_MODULES" "$BUNDLED_NODE" mobile/scripts/verify-mobile-renderer.mjs
    ran=1
  elif command -v node >/dev/null 2>&1 && node -e "import('playwright')" >/dev/null 2>&1; then
    echo "[agent-verify] mobile renderer"
    node mobile/scripts/verify-mobile-renderer.mjs
    ran=1
  else
    echo "[agent-verify] skip mobile renderer: playwright unavailable"
  fi
fi

if [ -x mobile/scripts/verify-release-readiness.sh ]; then
  echo "[agent-verify] mobile release readiness"
  mobile/scripts/verify-release-readiness.sh
  ran=1
fi

if [ "$ran" -eq 0 ]; then
  echo "[agent-verify] 未发现自动验证入口。本脚本已作为统一入口存在；项目任务中请补入测试、build、lint、golden sample 或 dry-run。"
  exit 1
fi
