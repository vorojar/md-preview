# 当前任务

## 目标

- 将桌面版升级到 `1.1.5`，吸收移动端经验：空白首页提供 Open File 和最近文件入口。
- 在桌面工具栏补齐打开文件与预览搜索，保留现有编辑、打印和更新检测。
- 同步官网、README、README_zh、CHANGELOG、bundle 版本，并走发布验证。

## 非目标

- 不重构渲染主线，不引入新前端框架或新运行时依赖。
- 不改变手机端已发布的 Android `1.0.6` 产物。

## 验收场景

- [x] 空白启动页含主操作 `Open File`，并在本机存在历史记录时展示最近文件。
- [x] 已加载文档时，工具栏含 Open / Find / Edit / Print，`Cmd/Ctrl+F` 可打开搜索栏。
- [x] 最近文件只写入本机配置目录，不进入仓库和发布包。
- [x] `./scripts/verify.sh` 通过。
- [ ] 桌面 `v1.1.5` GitHub Release 生成，并完成 macOS DMG 签名、公证、staple。

## 执行记录

- [x] 已批量理解桌面端 Rust/WebView 结构、文档、官网和 release workflow。
- [x] 已完成桌面启动页、最近文件、打开按钮、搜索按钮实现。
- [x] 已补充单测覆盖空态和工具栏入口。
- [x] 已同步文档与版本号。

## 验证记录

```text
命令：cargo test
结果：通过。8/8 tests passed。

命令：./scripts/verify.sh
结果：通过。cargo test 8/8；iOS xcodegen/build 与 Swift parse 通过；Android debug/release readiness 通过；mobile renderer 通过。

命令：MD_PREVIEW_BENCH=1 cargo run --quiet；MD_PREVIEW_BENCH=1 cargo run --quiet -- README.md
结果：通过。空态和文件态都收到 WebView ready 首帧信号。

命令：cargo build --release
结果：通过。桌面 release profile 构建成功。
```

## 风险和假设

- 预览搜索使用系统 WebView 的 `window.find`，保持轻量；不同平台查找高亮由系统 WebView 负责。
- 最近文件按本机路径保存，删除或移动后的文件会在下次加载最近列表时过滤。
