# AGENTS.md

本文件约束在 `hdsl-rs` 仓库中工作的自动化模型。开发必须遵循实施计划 `docs/developer/src/plan.md` 与根目录 `README.md`；本文件与实施计划冲突时，以实施计划为准。

## 项目边界

- `hdsl-rs` 是独立 Git 仓库。同级 `reference/`（HMCL、HMCL-rs、deepseek-harness、旧版 HDSL）只读参考，其源码、图标、壁纸与角色插画不得复制进本仓库或提交历史。
- 图形资源必须原创；保留 GPL-3.0-only 许可与必要的来源声明。
- 只管理上游 DeepSeek Harness，不修改其源码、依赖声明、profile patch 或启动逻辑。

## 文档先行（强制）

每项用户可见功能按固定顺序提交，且每个提交都可构建：

1. `docs(dev): ...`：先写行为、上游契约、失败路径与验收条件，页面状态标为“计划中”。
2. `feat:` 或 `fix:`：实现代码与测试，把对应开发者页面改为“已实现”或“部分已实现”。
3. `docs(user): ...`：补操作步骤、截图说明与排障。

- 一次提交只覆盖一个可回滚的功能切片，不得把整项功能压成一个大提交。
- 内部重构没有用户行为变化时可省略 `docs(user)`，但必须更新受影响的开发者文档。
- 提交前运行适用的格式、测试与文档构建检查。
- 发行版本打 Git 标签，保留分阶段历史以便回滚。
- 按此协议组织提交；未获指示时不做无关提交、不改写历史、不使用 `--no-verify` 或强制推送。

## Git 卫生

- 忽略构建产物、下载的运行时、实例数据与密钥；提交应用的 `Cargo.lock`。
- 不得提交密钥、`.credentials.yaml`、实例数据、下载的运行时或生成的书籍（`docs/**/book/`）。
- 提交信息沿用现有单行风格：`type: 摘要`（如 `feat:`、`fix:`、`docs(dev):`、`docs(user):`）。
- 参考项目不进入本仓库版本历史。

## 文档结构

- 两本简体中文优先的 mdBook：`docs/developer`（架构、数据格式、上游接口、兼容规则、构建与测试）与 `docs/user`（安装、实例、插件、API、排障）。
- 两本 `book.toml` 均设置 `create-missing = false`，不得让 mdBook 自动创建缺失页面。
- 每个开发者页面以“状态：计划中/已实现/部分已实现”开头，并随代码更新。
- 用户手册 HTML 随安装包与便携包提供；应用“帮助”按钮在系统浏览器打开本地副本，离线可用；Markdown 源文件保留在仓库。
- `docs/images/` 预留给 README 与两本手册的截图，完善文档时再引用，不要删除。

## 上游与安全约束

- 安装 Harness 只使用 npm 精确版本 `@deepseek-ai/dsh@<版本>`；启动使用该安装的 `bin` 入口、实例工作目录与实例专属 `DSH_HOME`。
- 插件操作使用该实例同一版本的 `dsh plugin --profile web`，不得使用系统全局 `dsh`。
- 不得为 `@deepseek-ai/*` 包写入 pnpm `overrides`；`allowBuilds` 仅在用户逐项批准构建脚本后写入。
- 网络来源限定为 Node.js 官方发行索引、npm registry、插件目录与 GitHub 元数据；安装前验证 Node 官方 SHA256 与 npm integrity。
- API 页只编辑已核验的 DeepSeek 官方字段；保存前验证，失败不得部分写入；密钥不得出现在日志或错误中。
- 插件只在具备明确 `@deepseek-ai/*` 依赖证据或人工维护兼容记录时提供安装；证据缺失或冲突时不提供。安装前停止实例并备份 profile，失败时恢复备份。

## 验证

在 `hdsl-rs` 目录运行与 CI 一致的检查：

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`（需要网络的测试标为 `#[ignore]`，用 `-- --ignored` 单独运行）
- `mdbook build docs/developer`、`mdbook test docs/developer`、`mdbook build docs/user`、`mdbook test docs/user`（mdBook 0.5.3）
- `python scripts/check_docs.py`
- 打包：Windows `scripts/package-windows.ps1`，Linux `bash scripts/package-linux.sh`

Windows GNU 目标链接需要 `shlwapi`：构建前把 `C:\msys64\mingw64\bin` 加入 `PATH`，否则链接器报 `ld: cannot find -lshlwapi`。
