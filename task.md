# 当前任务

## 目标

- 改善宽屏下多列 Markdown 表格的可读性：正文仍保持阅读宽度，多列表格自动使用更宽区域。
- 发布 `1.1.13`：完成验证、tag/release、macOS DMG 签名/公证/staple，并更新 Sparkle `appcast.xml`。

## 非目标

- 不把所有正文改成全宽自适应，避免普通段落在宽屏上变成长行。
- 不新增用户需要手动切换的布局设置；本次优先用自动表格规则解决截图中的痛点。
- 不调整 Windows/Linux 更新模型。

## 验收场景

- [x] 4 列及以上表格会被自动包进宽屏表格容器。
- [x] 普通正文仍保留阅读栏宽度。
- [x] 表头不再被窄列强行挤成竖排；超宽表格通过表格区域横向滚动处理。
- [x] 桌面和移动共享增强脚本保持一致。
- [x] `cargo test` 通过。
- [x] `./scripts/verify.sh` 通过。
- [ ] `v1.1.13` GitHub Release 完成，Release asset 包含 macOS DMG、Windows EXE、Linux tarball、`appcast.xml`。
- [ ] `./release-sign.sh v1.1.13` 完成，macOS DMG 和内层 app 已签名、公证、staple。

## 执行记录

- [x] 已确认现有 `#app max-width: 820px` 与 `table width: 100%` 会把多列表格限制在正文栏内，导致列头和单元格过度换行。
- [x] 已采用“默认阅读宽度 + 多列表格宽屏突破”的最小方案。
- [x] 已增加 `mdp-table-wrap` 增强逻辑，4 列及以上表格自动包装。
- [x] 已同步桌面 CSS、移动 CSS、桌面/移动增强脚本。
- [x] 已将版本号更新为 `1.1.13` 并记录 changelog。

## 验证记录

```text
命令：cargo test
结果：通过。10/10 tests passed。新增 page_expands_multi_column_tables 覆盖宽屏表格增强入口。

命令：./scripts/verify.sh
结果：通过。guard、cargo test、macOS Sparkle 验证、Windows self-update 验证、iOS build/parse、Android debug/release、mobile renderer/release readiness 均通过。
```

## 风险和假设

- 表格规则按列数触发：4 列及以上视为需要更宽布局；少列表格保持原有正文内布局。
- 浏览器插件安全策略拦截了 `data:` 和 `file://` 本地检查页，因此本次视觉验证以单元测试和项目统一验证为准，未保留浏览器截图。
