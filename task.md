# 当前任务

## 目标

- 将 Windows 版补齐到和 macOS 一样的应用内自更新体验。
- 发布 `1.1.11`：包含 Windows 安装器、WinSparkle appcast、文档和官网同步。
- 保留 Windows ZIP 便携包，避免破坏已有手动下载路径。

## 非目标

- 不引入 Electron/Tauri 或更换桌面 WebView 架构。
- 不处理 Windows Authenticode 证书签名；当前仓库/签名链路没有 Windows 代码签名证书。

## 验收场景

- [x] Windows release 包含 `MD-Preview-windows-x64-Setup.exe`。
- [x] Windows 安装器包含 `md-preview.exe`、`WinSparkle.dll`、开始菜单入口、卸载项和 Markdown 打开方式注册。
- [x] Windows 应用内更新走 WinSparkle，读取 `appcast-windows.xml`，并使用 EdDSA 公钥校验更新包。
- [x] macOS Sparkle 更新能力保持不回退。
- [x] `./scripts/verify.sh` 通过。
- [ ] `v1.1.11` GitHub Release 完成，Release asset 包含 macOS DMG、Windows 安装器、Windows ZIP、Linux tarball、`appcast.xml`、`appcast-windows.xml`。

## 执行记录

- [x] 已确认 WinSparkle 官方要求：随应用分发 `WinSparkle.dll`，appcast 走 HTTPS，Windows 安装器可通过 `sparkle:installerArguments="/S"` 静默执行。
- [x] 已完成 Windows WinSparkle 原生桥接、安装器、release workflow 和验证脚本。
- [x] 已同步 README、README_zh、官网和 CHANGELOG。

## 验证记录

```text
命令：cargo test
结果：通过。9/9 tests passed。

命令：cargo check --target x86_64-pc-windows-gnu
结果：通过。Windows cfg 下 WinSparkle FFI 编译通过。

命令：scripts/verify-windows-update.sh
结果：通过。确认 WinSparkle DLL、安装器脚本、Windows appcast EdDSA 签名格式。

命令：makensis -DVERSION=1.1.11 windows/installer.nsi
结果：通过。本机用 NSIS 3.11 成功编译 Windows 安装器脚本。

命令：./scripts/verify.sh
结果：通过。guard、cargo test、macOS Sparkle 验证、WinSparkle 验证、iOS build/parse、Android debug/release、mobile renderer/release readiness 均通过。
```

## 风险和假设

- Windows 自更新会校验 EdDSA appcast 签名；安装器本身目前未做 Authenticode 签名，因为项目没有配置 Windows 代码签名证书。
- WinSparkle 更新前若检测到源码编辑区有未保存修改，会拒绝让更新器直接关闭应用，避免丢内容。
