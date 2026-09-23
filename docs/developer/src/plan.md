# HDSL Rust 启动器实施计划

状态：首版进行中。本页是首版的总览与边界；各子系统的详细设计与验收条件见后续页面。

## 目标与项目边界

新建独立的 `hdsl-rs` Git 仓库，以 Rust 和 Slint 开发 Windows、Linux 启动器。首版完成运行时托管、Harness 版本与实例管理、兼容插件管理及 Web 启动；模型路由与凭据由实例的 Harness 本体管理。使用全新数据目录；暂不导入 Java 版 HDSL 的数据。

## 界面与核心能力

- 按原版 HMCL 的页面层级、布局和动效重做首页、安装向导、实例页与设置页；替换为 Harness 内容。图标和角色插画原创设计，不复制 HMCL、HMCL-rs 或 DeepSeek 娘的资源。保留 GPLv3 许可及必要的来源声明。[HMCL 许可说明](https://github.com/HMCL-dev/HMCL#license)
- 界面复刻的度量、组件库、页面映射、分阶段切片与子智能体分工见 [UI 复刻计划：对齐 HMCL](ui-parity.md)；待生成的原创图形资源见 [UI 图形资源清单](ui-assets.md)。设置页命名按需求调整：全局游戏设置 → 全局设置，Java 管理 → 环境管理，通用、外观、下载保持不变。
- 自动下载并校验 Node.js 24 LTS、pnpm 11.7.0 和指定的 Harness 精确版本。每个实例固定版本、工作目录、端口和独立 `DSH_HOME`，支持启动、停止与日志查看。升级创建新实例，保留旧实例及其 home。[Harness CLI 参考](https://github.com/deepseek-ai/deepseek-harness/blob/master/apps/cli/reference/README.md)
- 模型路由、API Key 与端点由每个实例的 Harness 本体管理；启动器不提供这些编辑入口。“设置”页按 HMCL 版式实现，含「全局设置 / 环境管理 / 通用 / 外观 / 下载 / 帮助 / 反馈 / 关于」，但不含任何写入 Harness 凭据或模型设置的入口。第三方认证交给插件界面。[官方模型设置](https://deepseek-harness.github.io/deepseek-harness/en/guide/providers)

## 插件目录与兼容性

- 以 [DSH Marketplace 目录](https://github.com/DshMarketPlace/dshmarketplace)发现候选，核验 GitHub 来源、安装包和 `dsh.bundle` 声明。首批人工审查 DSH Plugin Marketplace、DSH Market、DSH Better Sidebar 和 OpenViking 插件。
- 只收录具有明确 `@deepseek-ai/*` 版本依赖声明，或列入项目人工维护兼容记录的插件版本。按实例实际安装的 Harness 包版本匹配依赖范围；证据缺失或明确冲突时不提供安装。默认推荐最新兼容版本。
- 安装前备份并停止目标 profile；使用官方 `dsh plugin` 安装，随后检查配置能否启动。失败时恢复备份。构建脚本逐次展示并由用户确认。

## Git 与文档工作流

- `hdsl-rs` 单独初始化 Git；参考项目不纳入其版本历史。忽略构建产物、下载的运行时、实例数据与密钥，提交应用的 `Cargo.lock`。
- 建立简体中文优先的两本 mdBook：**开发者手册**记录架构、数据格式、上游接口、兼容规则、构建与测试；**用户手册**覆盖安装、实例、插件、设置和排障。设置 `create-missing = false`，CI 构建两本手册并检查内部链接及可测试代码示例。[mdBook 文档](https://rust-lang.github.io/mdBook/guide/creating.html)
- 每项用户功能按固定顺序提交：`docs(dev)` 设计与验收条件 → `feat`/`fix` 代码和测试 → `docs(user)` 操作说明。设计文档在实现前标为“计划中”，代码完成时改为“已实现”。每个提交保持可构建；按单一功能或子系统拆分提交，不压成一个大提交。发布版本打 Git 标签，保留提交历史以便回滚。
- 用户手册 HTML 随安装包和便携包提供；应用“帮助”按钮在系统浏览器打开本地副本，离线可用。Markdown 源文件保留在仓库。

## 验收与发行

在 Windows x64、Linux x64/arm64 的干净环境验证下载校验、双版本隔离运行、兼容插件安装与失败回滚，并确认密钥不进入日志。CI 检查 Rust 代码、两本手册及发行包；交付 Windows 安装器和 ZIP、Linux `.deb` 和便携压缩包。

**首版假设：**图形启动以官方 `web` profile 为主；会话迁移、整合包和旧版数据导入留待后续版本。
