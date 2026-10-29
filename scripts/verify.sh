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

echo "[agent-verify] distribution policy"
bash -n release-sign.sh scripts/release.sh bundle.sh hooks/pre-push
python3 scripts/verify-distribution.py
ran=1

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
for marker in ("new-file", "AUTOSAVE_DEBOUNCE_MS = 700", "window.__mdPreviewResolveExternalChange", "UserEvent::Quit"):
    if marker not in src:
        raise SystemExit(f"desktop New Markdown/autosave contract is missing: {marker}")
PY
  ran=1
fi

if [ -f docs/index.html ] && [ -f README.md ] && [ -f README_zh.md ]; then
  echo "[agent-verify] current product story"
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
for marker in ("Multi-document tabs", "Session restore", "Cross-platform"):
    if marker not in site:
        raise SystemExit(f"website is missing v1.2 product marker: {marker}")
for marker in ("Desktop tabs", "Session restore", "Finder workflow"):
    if marker not in readme:
        raise SystemExit(f"README.md is missing v1.2 product marker: {marker}")
for marker in ("桌面标签", "会话恢复", "Finder 工作流"):
    if marker not in readme_zh:
        raise SystemExit(f"README_zh.md is missing v1.2 product marker: {marker}")
for marker in ("New in v1.4", "Local document links", "Counts, zoom, continuity", "Reliable autosave"):
    if marker not in site:
        raise SystemExit(f"website is missing current product marker: {marker}")
for marker in ("Local document links", "Live statistics", "Content zoom", "Scroll continuity", "Reliable autosave"):
    if marker not in readme:
        raise SystemExit(f"README.md is missing current product marker: {marker}")
for marker in ("本地文档链接", "实时统计", "正文缩放", "滚动连续", "可靠自动保存"):
    if marker not in readme_zh:
        raise SystemExit(f"README_zh.md is missing current product marker: {marker}")
if "What's New" not in src or "Follow local document links" not in src:
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

if [ -f scripts/verify-desktop-reading-tools.mjs ]; then
  BUNDLED_NODE="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
  BUNDLED_NODE_MODULES="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules"
  if [ -x "$BUNDLED_NODE" ] && [ -d "$BUNDLED_NODE_MODULES" ]; then
    echo "[agent-verify] desktop reading tools"
    NODE_PATH="$BUNDLED_NODE_MODULES" "$BUNDLED_NODE" scripts/verify-desktop-reading-tools.mjs
    ran=1
  elif command -v node >/dev/null 2>&1 && node -e "import('playwright')" >/dev/null 2>&1; then
    echo "[agent-verify] desktop reading tools"
    node scripts/verify-desktop-reading-tools.mjs
    ran=1
  else
    echo "[agent-verify] skip desktop reading tools: playwright unavailable"
  fi
fi

if [ -f scripts/verify-desktop-tab-progress.mjs ]; then
  BUNDLED_NODE="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node"
  BUNDLED_NODE_MODULES="/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules"
  if [ -x "$BUNDLED_NODE" ] && [ -d "$BUNDLED_NODE_MODULES" ]; then
    echo "[agent-verify] desktop tab progress"
    NODE_PATH="$BUNDLED_NODE_MODULES" "$BUNDLED_NODE" scripts/verify-desktop-tab-progress.mjs
    ran=1
  elif command -v node >/dev/null 2>&1 && node -e "import('playwright')" >/dev/null 2>&1; then
    echo "[agent-verify] desktop tab progress"
    node scripts/verify-desktop-tab-progress.mjs
    ran=1
  else
    echo "[agent-verify] skip desktop tab progress: playwright unavailable"
  fi
fi

if [ -x scripts/verify-windows-self-update.sh ]; then
  echo "[agent-verify] Windows self-update"
  bash scripts/verify-windows-self-update.sh
  ran=1
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
