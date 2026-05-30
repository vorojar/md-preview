# 项目协作规则

## 项目目标

- MD Preview 是本地优先 Markdown 预览器，核心场景是快速打开 AI 生成文档、README、计划文档和手机收到的 Markdown 文件。
- 桌面端保持 Rust + 系统 WebView 的轻量路线；手机端 MVP 聚焦只读快速预览和系统文件打开入口。

## 技术栈与入口

- 主要语言/框架：Rust (`wry`/`tao`)、iOS UIKit + `WKWebView`、Android Java + `WebView`。
- 桌面入口：`src/main.rs`。
- 手机共享渲染层：`mobile/shared/preview.html`。
- iOS 工程：`mobile/ios/project.yml`，用 XcodeGen 生成。
- Android 工程：`mobile/android` Gradle project。
- 测试入口：优先使用 `scripts/verify.sh`。
- 常用命令：

```bash
./scripts/verify.sh
cargo test
cd mobile/ios && xcodegen generate
cd mobile/android && gradle :app:assembleDebug
```

## 工作方式

- 先批量理解，再直接实现，最后集中验证和汇报。
- 修改前先找同类实现和现有命名风格。
- 改动保持聚焦，每一行 diff 都要对应当前任务目标、缺陷修复或验证需要。
- 不顺手升级依赖，不做无关重构，不删除用户已有改动。
- 同一事实只保留一个来源；提示词、配置、常量、接口和文档不要重复定义。
- 能用脚本、schema、测试、hook 或硬拦截保证的，不交给模型自律。

## 验收标准

- 每个任务开始前明确目标、非目标和至少 3 个验收场景。
- 修 bug 尽量先写最小复现或测试，再修复，再验证通过。
- 测试必须验证具体业务结果，禁止只检查 `toBeDefined()`、`toBeTruthy()`。
- UI 任务必须验证桌面和移动端关键视口，检查文字溢出、遮挡、安全区和交互状态。
- 数据/内容任务必须给出样例输入、样例输出和失败样例。
- 发布/打包任务必须有 dry-run 或 checklist。

## 错误处理

- 不要用 `value ?? 0`、`name || "Unknown"` 这类 fallback 掩盖不该为空的业务数据。
- 业务逻辑层不要滥用 `try/catch` 吞错；让错误自然暴露，只在 API、任务或命令入口边界统一处理。
- 改输入校验前先列真实输入变体和边界值，再改实现。

## 安全边界

- API key、token、账号密码只放 `.env` 或系统凭据，不写入 README、日志、注释、commit message 或对话总结。
- 对外发布、删除数据、付费、账号权限变更必须先确认。
- `.env`、日志、截图、导出数据提交前要检查敏感信息。
- 涉及删除、覆盖工作区、递归清理、`git reset --hard`、`git clean`、外发敏感文件时必须确认。

## 工作区卫生

- 交付前检查 `git status`；提交前检查 `git diff` 或 `git diff --cached`。
- 一个 commit 只做一件事；不要把 feature、fix、格式化和无关清理混在一起。
- 项目内临时文件放项目约定目录；不要在仓库根目录散落一次性文件。

## 交付要求

最终回复只保留高信号内容：

- 改了什么。
- 验证命令和结果。
- 风险、假设或无法验证的部分。
- 对效率和返工减少的具体影响。
