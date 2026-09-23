# HDSL

HDSL 是用 Rust 与 [Slint](https://slint.dev/) 编写的 DeepSeek Harness 桌面启动器。它托管 Node.js 运行时，按实例安装、隔离并启动精确版本的 `@deepseek-ai/dsh`，并管理兼容插件。界面按 HMCL 的页面层级、布局与动效复刻，内容替换为 DeepSeek Harness。

此仓库独立于同级参考项目；参考项目（HMCL、HMCL-rs、deepseek-harness 等）的源码、图标、壁纸与插画均不进入本仓库。

## 状态

首版开发中，界面复刻 S0–S7 已落地：

- 无边框窗口外壳、HMCL 设计令牌与组件库（`crates/ui/ui/theme.slint`、`components/`）。
- 页面：首页、实例列表、下载与安装、实例详情、设置、日志与对话框（`crates/ui/ui/pages/`）。
- 动效：壁纸背景、页面淡入、按下水波纹、窗口开合。
- 资源：原创 Roxy Bot 标识、壁纸与立绘已入库；单色图标当前为原创占位图。

待完成：设置持久化、主题与背景切换实际生效、插件与下载真实数据接线、实例图标选择对话框、日志逐行着色。详见[开发者手册](docs/developer/src/README.md)的 [UI 页面设计明细](docs/developer/src/ui-pages.md) 与 [实施计划](docs/developer/src/plan.md)。

## 功能

- 从 npm 官方目录读取 `@deepseek-ai/dsh` 全部已发布版本（含 alpha、beta、rc 预览版），按精确版本安装并校验完整性。
- 自动下载并校验 Node.js 24 LTS 与 pnpm；每个实例固定版本、工作目录、端口与独立 `DSH_HOME`。
- 启动、停止实例并查看运行日志；升级创建新实例，保留旧实例及其 home。
- 只收录具备明确 `@deepseek-ai/*` 依赖证据的兼容插件，安装前备份并停止 profile，失败时回滚。

模型路由、API Key 与端点由每个实例的 Harness 本体管理，启动器不提供写入入口，也不改写实例的 `.credentials.yaml` 与 `settings.yaml`。

## 构建与运行

需要 Rust（edition 2024）与 Cargo。

```powershell
cargo run -p hdsl
```

Windows GNU 目标链接需要 `shlwapi`，构建前把 mingw 加入 `PATH`：

```powershell
$env:PATH = "C:\msys64\mingw64\bin;$env:PATH"
cargo build -p hdsl
```

## 验证

在仓库根目录运行与 CI 一致的检查：

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked          # 需要网络的测试标为 ignored，用 -- --ignored 单独运行
mdbook build docs/developer; mdbook test docs/developer
mdbook build docs/user;      mdbook test docs/user      # mdBook 0.5.3
python scripts/check_docs.py
```

打包：Windows 用 `scripts/package-windows.ps1`，Linux 用 `bash scripts/package-linux.sh`。

## 目录结构

```
crates/core   磁盘状态、下载、进程、npm 元数据、插件兼容与日志脱敏（不依赖 Slint）
crates/ui     Slint 界面：theme.slint、components/、pages/、icons.slint、assets/
crates/app    可执行程序 hdsl，把界面事件接到异步服务
docs/developer  开发者手册（架构、数据格式、上游接口、构建与测试）
docs/user       用户手册（安装、实例、插件、设置、排障）
scripts         资源生成/覆盖、文档检查与打包脚本
```

## 图形资源

图形资源必须原创，不复制参考项目的图标、壁纸或插画。`crates/ui/ui/assets/` 中的占位图与原创标识由脚本生成或提交；开发期可用 `scripts/stage-candidate-icons.ps1` 把第三方候选图标覆盖到 `assets/icons/` 校对，但**提交前必须还原**（`git checkout -- crates/ui/ui/assets/icons`）。详见 [UI 图形资源清单](docs/developer/src/ui-assets.md)。

## 文档

- 开发者手册：[docs/developer/src/README.md](docs/developer/src/README.md)
- 用户手册：[docs/user/src/README.md](docs/user/src/README.md)
- 界面复刻：[UI 复刻计划：对齐 HMCL](docs/developer/src/ui-parity.md)、[UI 页面设计明细](docs/developer/src/ui-pages.md)
- 实施计划：[plan.md](docs/developer/src/plan.md)
- 提交与文档协议：[workflow.md](docs/developer/src/workflow.md)

用户手册 HTML 随安装包与便携包提供，应用「帮助」按钮在系统浏览器打开本地副本，离线可用。

## 许可

本项目以 GPL-3.0-only 发布，见 [LICENSE](LICENSE) 与 [NOTICE](NOTICE)。HDSL 并非 HMCL 或 DeepSeek AI 的官方产品；界面借鉴 HMCL 的页面组织方式，图形资源由本项目独立创作。
