# Changelog

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
