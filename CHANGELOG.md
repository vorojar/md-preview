# Changelog

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
