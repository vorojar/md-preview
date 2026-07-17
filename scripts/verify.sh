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

if [ -f release-sign.sh ]; then
  echo "[agent-verify] release signing contract"
  bash -n release-sign.sh scripts/release.sh
  if ! grep -F 'remote-mac-sign/sign.sh' release-sign.sh >/dev/null; then
    echo "[agent-verify] release-sign.sh must use the local-first remote-mac-sign/sign.sh entrypoint" >&2
    exit 1
  fi
  if grep -F 'SIGN_SCRIPT="$HOME/.claude/skills/remote-mac-sign/sign_remote.sh"' release-sign.sh >/dev/null; then
    echo "[agent-verify] release-sign.sh must not hard-code the remote-only signer" >&2
    exit 1
  fi
  ran=1
fi

if [ -f src/main.rs ]; then
  echo "[agent-verify] desktop print stylesheet"
  python3 - <<'PY'
from pathlib import Path
src = Path("src/main.rs").read_text()
if "@page {{\n  margin: 12mm;\n}}" not in src:
    raise SystemExit("src/main.rs must set @page margin: 12mm for native desktop printing")
if "@media print {{" not in src or "#app {{ max-width: none; padding: 0; }}" not in src:
    raise SystemExit("src/main.rs must keep print media rules focused on preview output")
PY
  echo "[agent-verify] desktop tabs and macOS close-tab shortcut"
  python3 - <<'PY'
from pathlib import Path
src = Path("src/main.rs").read_text()
session = Path("src/session.rs").read_text()
if 'id="tabbar"' not in src or "window.__setTabs" not in src:
    raise SystemExit("desktop page must expose the top tab bar and deterministic tab renderer")
if "session.json" not in src or "PersistedSession" not in session:
    raise SystemExit("desktop tabs must persist a dedicated session separate from Recent")
if '"Close Tab"' not in src or "mdPreviewCloseTab:" not in src:
    raise SystemExit("macOS File menu must expose Close Tab through the tab state machine")
if '"w",\n        NSEventModifierFlags::Command' not in src:
    raise SystemExit("macOS Close Tab must keep Cmd+W")
if "window.__setMissing" not in src or "data-locate-tab" not in src:
    raise SystemExit("missing session files must keep their tab and offer relocation")
PY
  ran=1
fi

if [ -f docs/index.html ] && [ -f README.md ] && [ -f README_zh.md ]; then
  echo "[agent-verify] v1.2 product story"
  python3 - <<'PY'
from pathlib import Path
import tomllib

version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
site = Path("docs/index.html").read_text()
readme = Path("README.md").read_text()
readme_zh = Path("README_zh.md").read_text()
src = Path("src/main.rs").read_text()

if f'"softwareVersion": "{version}"' not in site:
    raise SystemExit("website structured data must match Cargo.toml version")
for marker in ("Multi-document tabs", "Session restore", "Finder to source edit"):
    if marker not in site:
        raise SystemExit(f"website is missing v1.2 product marker: {marker}")
for marker in ("Desktop tabs", "Session restore", "Finder workflow"):
    if marker not in readme:
        raise SystemExit(f"README.md is missing v1.2 product marker: {marker}")
for marker in ("桌面标签", "会话恢复", "Finder 工作流"):
    if marker not in readme_zh:
        raise SystemExit(f"README_zh.md is missing v1.2 product marker: {marker}")
if "What's New" not in src or "resume them across launches" not in src:
    raise SystemExit("macOS About must keep the current product positioning and What's New entry")
PY
  ran=1
fi

if [ -f macos/finder-extension/FinderSyncExtension.swift ]; then
  echo "[agent-verify] Finder Sync integration contract"
  grep -F 'com.apple.FinderSync' macos/finder-extension/project.yml >/dev/null
  grep -F 'mdpreview' macos/finder-extension/FinderSyncExtension.swift >/dev/null
  grep -F 'MDPreviewFinderExtension.appex' bundle.sh >/dev/null
  grep -F 'CFBundleURLTypes' bundle.sh >/dev/null
  ran=1
fi

if [ -f Cargo.toml ]; then
  if command -v cargo >/dev/null 2>&1; then
    echo "[agent-verify] cargo test"
    cargo test
    ran=1
  fi
fi

if [ -f scripts/verify-anchor-navigation.mjs ]; then
  BUNDLED_NODE="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
  BUNDLED_NODE_MODULES="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules"
  if [ -x "$BUNDLED_NODE" ] && [ -d "$BUNDLED_NODE_MODULES" ]; then
    echo "[agent-verify] anchor navigation"
    NODE_PATH="$BUNDLED_NODE_MODULES" "$BUNDLED_NODE" scripts/verify-anchor-navigation.mjs
    ran=1
  elif command -v node >/dev/null 2>&1 && node -e "import('playwright')" >/dev/null 2>&1; then
    echo "[agent-verify] anchor navigation"
    node scripts/verify-anchor-navigation.mjs
    ran=1
  else
    echo "[agent-verify] skip anchor navigation: playwright unavailable"
  fi
fi

if [ -f scripts/verify-desktop-search.mjs ]; then
  BUNDLED_NODE="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
  BUNDLED_NODE_MODULES="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules"
  if [ -x "$BUNDLED_NODE" ] && [ -d "$BUNDLED_NODE_MODULES" ]; then
    echo "[agent-verify] desktop search"
    NODE_PATH="$BUNDLED_NODE_MODULES" "$BUNDLED_NODE" scripts/verify-desktop-search.mjs
    ran=1
  elif command -v node >/dev/null 2>&1 && node -e "import('playwright')" >/dev/null 2>&1; then
    echo "[agent-verify] desktop search"
    node scripts/verify-desktop-search.mjs
    ran=1
  else
    echo "[agent-verify] skip desktop search: playwright unavailable"
  fi
fi

if [ -x scripts/verify-sparkle-update.sh ]; then
  echo "[agent-verify] Sparkle update"
  scripts/verify-sparkle-update.sh
  ran=1
fi

if [ -x scripts/verify-windows-self-update.sh ]; then
  echo "[agent-verify] Windows self-update"
  bash scripts/verify-windows-self-update.sh
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
