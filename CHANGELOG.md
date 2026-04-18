# Changelog

## 0.3.8

- 工具栏按钮在编辑/预览模式间的位置漂移修复：`html { overflow-y: scroll; scrollbar-gutter: stable; }` 让滚动条空间永久预留，两种状态下 viewport 宽度一致，`position: fixed; right` 的定位不再因滚动条出现/消失而偏移
- 编辑模式下隐藏打印按钮：打印属于"读完"场景，与编辑心智冲突；Cmd/Ctrl+P 系统快捷键不受影响

## 0.3.7

- 工具栏改为 hover 时浮现：鼠标离开窗口自动淡出，不干扰阅读
- 空态（未打开文件）不再显示工具栏，避免"没内容可编辑/打印"的按钮误导
- 编辑模式切换为全宽布局：textarea 撑满窗口，提供更开阔的写作空间；预览仍保持 820px 居中阅读；模式差异成为切换的视觉反馈

## 0.3.6

- 新增 Source 编辑模式：右上角浮动按钮切换预览/编辑，编辑模式下 textarea 装 md 原文，打字即生效；Cmd/Ctrl+E 也可切换
- Cmd/Ctrl+S 保存改动，watcher 500ms 自写抑制避免因保存触发无谓重渲染；脏态在标题栏以「• 」前缀显示
- 新增打印按钮：Cmd/Ctrl+P 或工具栏右侧按钮调起系统打印；打印样式自动隐藏工具栏与编辑区，只打印预览内容
- 按钮使用内联 SVG 图标（非 emoji），明暗模式适配；tooltip 展示快捷键
- 保存后仅更新预览区（不触碰 textarea 光标位置）；外部改动走完整 `__setContent` 同步两者

## 0.3.5

- Windows 首次启动不再弹窗询问是否跳转「设置 › 默认应用」；后台静默写入注册表后即结束，用户按需在 .md 文件右键选 MD Preview 即可

## 0.3.4

- 窗口首次启动居中显示（计算 primary monitor 的可见区域）
- 记住窗口位置/大小：关闭时写入 `<config>/window.geom`，下次启动恢复；跨屏幕断开场景下自动回退居中，避免窗口飘到不可见区
- 空态提示与 Windows 首次启动弹窗按系统语言切换（`zh*` → 中文，其它 → 英文，基于 `sys-locale`）
- Windows：WebView2 的 UserDataFolder 迁移到 `%LOCALAPPDATA%\md-preview\WebView2`，exe 同目录不再产生 `*.WebView2/` 缓存目录
- 统一 `config_dir()` 跨平台：Windows 使用 `%LOCALAPPDATA%`，macOS/Linux 使用 `~/.config/md-preview`

## 0.3.3

- 首屏渲染加速：highlight.js（119KB）从同步 inline 改为 `<script type="text/x-hljs">` 的惰性文本，首次 paint 不再为 hljs 解析/执行等待；在 `requestIdleCallback` 里异步求值并上色，中低端机首屏阻塞从约 150ms 降至约 30ms
- 代码块首屏先以纯文本显示，~50ms 后自动着色（感知瞬间）
- 文件切换路径同步改走 idle 上色，避免对首次 idle callback 的时序竞态

## 0.3.2

- Windows：首次启动写 HKCU 注册表（.md/.markdown/.mdown/.mkd 的 OpenWithProgids + ProgID 定义 + Applications 条目），并弹窗引导用户去「设置 › 默认应用」完成关联（Win10+ 禁止应用静默设为默认）
- Windows：窗口左上角 HICON 现在使用应用图标（此前为系统默认）
- Windows：图标改为"白底粗体 #"风格，与 macOS 视觉一致；`gen_ico.py` 独立绘制每个尺寸（16/32/48/64/128/256），小尺寸不再因降采样而糊
- 所有平台：md 中的 http/https/mailto 链接改为通过系统默认浏览器/邮件客户端打开，不再劫持在 WebView 内导航
- 新增依赖：`image`（png+ico 解码，用于窗口图标）+ `winreg`（Windows-only）

## 0.3.1

- Windows: release 构建关闭 console subsystem，双击/Finder 式启动不再弹黑色 cmd 窗口 (#2)
- Windows: 通过 `winresource` 在 `.exe` 中嵌入应用图标（`assets/icon.ico`）
- `gen_icon.py` 同时产出 macOS `.icns` 与 Windows 多尺寸 `.ico`（16/32/48/64/128/256）

## 0.3.0

- 极简应用图标（白底 "#"，简洁干净）
- CLI 参数：`md-preview file.md` 直接打开
- 离线代码语法高亮（自研 2KB mini highlighter，替代 119KB highlight.js）
- 支持关键词、字符串、注释、数字、函数名着色
- 暗色/亮色主题自动切换
- 刷新保持滚动位置
- 去掉 image crate，二进制 986KB（< 1MB）
- macOS .app 打包脚本（`bundle.sh`）

## 0.1.0

- 初始版本
- 支持拖放 .md 文件预览
- 支持 Cmd/Ctrl+O 打开文件
- 文件修改自动刷新
- 支持 GFM 表格、任务列表、删除线
- 暗色模式自适应
- Release 二进制仅 ~970KB
