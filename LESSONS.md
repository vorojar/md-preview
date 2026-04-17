# LESSONS

项目踩坑记录，按时间倒序。接手人防雷用。

---

## 2026-04-17 — Windows 平台适配

### Cargo 的 `[target.'cfg(...)'.build-dependencies]` 按 **host** 评估

初次把 `winresource` 配成：

```toml
[target.'cfg(windows)'.build-dependencies]
winresource = "0.1"
```

又想靠 `CARGO_CFG_TARGET_OS` 运行时判断在 build.rs 里 `use winresource`。

失败原因：Cargo 的 `[target.<cfg>]` 表达式对 `build-dependencies` 评估的是 **host**（因为 build.rs 在 host 上运行），不是 target。macOS host 上 winresource 根本不会被 resolve，build.rs 里的符号会找不到，编译失败。

正确做法：**build.rs 自己**用 `#[cfg(target_os = "windows")]` 守护（检查的是 build.rs 编译时的 host）：

```rust
#[cfg(target_os = "windows")]
fn main() { winresource::WindowsResource::new().set_icon("assets/icon.ico").compile().unwrap(); }

#[cfg(not(target_os = "windows"))]
fn main() {}
```

Cargo.toml 那条 target-scoped build-dep 可保留也可删（保留后 mac 构建会 resolve 但不 compile），但 build.rs 的 host-cfg 守护是必须的。

### Commit 写 `Closes #N` → push 到默认分支时立即关 issue，**不是**等 release

这次 `fix(...) Closes #2` 的 commit 一 push master，GitHub 就自动关了 issue，比我打 tag + 发 release + 回复 issue 的流程早 10 分钟。导致我"先回复、再关"的意图和事实次序错位。

若希望**等 release 实测完成再关**，commit message 里就不能带 `Closes` / `Fixes` / `Resolves` / `Closed` / `Fixed` / `Resolved` 关键字，用 `Refs #N` 代替，最后手工 `gh issue close N` 控制关闭时机。

### 没有 Windows 机时验证 Windows 修复的完整链路

手上只有 macOS 时，Windows-only 行为（subsystem、资源嵌入）无法本地跑验，必须让 CI 做**真实验证**，不能只看 build 是否成功：

- CI 在 `windows-latest` runner 上 `cargo build --release`
- PowerShell 读 PE header 校验 `subsystem` 字段：`2` = Windows GUI，`3` = Console
  - 偏移：`0x3C` → PE 头偏移；再 +`0x5C` → IMAGE_OPTIONAL_HEADER.Subsystem (u16, LE)
- 用 Latin1 把 exe 字节串化后 `IndexOf` PNG 签名 `\x89PNG\r\n\x1a\n`，计数 ≥ 我们 ico 里 PNG 图像数量 → winresource 嵌入成功
- 发 release 后再 `gh release download` 拉下 zip 本机跑一遍同样校验，三重确认

这套已固化在 `.github/workflows/ci.yml`。任何回退会直接挂 CI。

### 本地 `block-dangerous.sh` hook 会拦几类命令

`~/.claude/hooks/block-dangerous.sh` 拦截规则（grep -E）：

```
rm\s+-rf\s+/ | push\s+--force | push\s+-f\s | reset\s+--hard | drop\s+table | clean\s+-fd
```

**不要试图绕过**。需要这些操作时把完整命令贴出来请用户手动在终端跑。

---

## 2026-04-17 — Vditor WYSIWYG 尝试并回撤

用户要求"预览和编辑同一状态"的轻量编辑体验。做了 Vditor IR 模式完整集成（+335KB 离线资源、+100KB 二进制、IPC save / dirty 指示 / watcher 自写抑制全套），CI 通过，但用户实测后判断"效果不好"，reset --hard + force push 整段历史抹掉。

教训：

- **"所见即所得" markdown ≠ 纯渲染器 + 一点编辑**。内在复杂度必有：光标/选区管理、增量解析、撤销栈、脏态、脏态与 watcher 冲突、文件保存/冲突。不存在"基本一样"的轻量 WYSIWYG。下次遇到"加个编辑功能很简单吧"的判断要先用这条挡一下。
- **风险较大的改动 push 前先让用户实测**。这次如果没 push，撤回就是本地 `git reset` 的事，不会触发 force push 和本地 hook。今后大改动推 commit 不推 push，等用户点头。
