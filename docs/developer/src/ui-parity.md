# UI 复刻计划：对齐 HMCL

状态：部分已实现。S0–S6 已落地：设计令牌、组件库、无边框窗口外壳，以及首页、实例列表、下载与安装、实例详情、设置、日志与对话框六个页面；S7 已完成壁纸、页面淡入与刷新接线，逐项水波纹动效与像素校对待续。页面度量与验收见 [UI 页面设计明细](ui-pages.md)，图形资源清单见 [UI 图形资源清单](ui-assets.md)。实施顺序遵循 [文档先行与 Git 工作流](workflow.md)，边界遵循 [架构与上游边界](architecture.md)。

## 目标与非目标

目标：

- 页面层级、区域布局、尺寸、间距、圆角、阴影、配色、字号与动效参数对齐 HMCL 默认主题（`hmcl.default` + `blue.css`）。
- 内容替换为 DeepSeek Harness：实例 = Harness 实例，版本 = Harness 版本，插件 = 兼容插件。
- 图形资源全部原创，文件名与尺寸对齐 HMCL 对应项，见 [UI 图形资源清单](ui-assets.md)。

非目标（首版）：

- 不实现 HMCL 的 Minecraft 专属功能：模组、资源包、世界、原理图、NBT 编辑器、多人联机、账户与皮肤。
- 不实现主题包市场、主题色生成器、字体选择等外观高级项；外观页先复刻版式并只接可落地的设置项。

## 设计令牌（从 HMCL 移植）

HMCL 的颜色来自 Material 3 角色变量 `-monet-*`，默认主题为 `blue.css` 的静态调色板，种子色 `#5C6BC0`。Slint 无 CSS，改为在 `ui/theme.slint` 导出 `Theme` 全局结构，供组件读取。

| 类别 | 取值 |
| --- | --- |
| 颜色角色 | 50 个 `-monet-*` 角色 + `primary-seed` + 9 个透明变体，见下表 |
| 字号 | 10 / 12 / 13 / 14 / 15 / 16 / 18 / 20 / 22 |
| 圆角 | 2 / 3 / 4 / 5 / 6 / 20 / 25 / 50 |
| 间距 | 0 / 2 / 4 / 5 / 6 / 8 / 10 / 12 / 16 / 22 / 24 |
| 阴影 | `depth-0`…`depth-5`，`card`、`options-list`、`card-non-transparent` |
| 动效 | `SHORT1..4`=50/100/150/200ms；`MEDIUM1..4`=250/300/350/400ms；`LONG1..4`=450/500/550/600ms；曲线 `EASE`/`EASE_IN`/`EASE_OUT`/`EASE_IN_OUT`/`STANDARD`/`EMPHASIZED_*` |

默认主题关键色（`blue.css`）：

| 令牌 | 值 | 令牌 | 值 |
| --- | --- | --- | --- |
| `primary` | `#4352A5` | `on-primary` | `#FFFFFF` |
| `primary-container` | `#5C6BC0` | `on-primary-container` | `#F8F6FF` |
| `secondary-container` | `#D0D5FD` | `on-secondary-container` | `#565B7D` |
| `tertiary` | `#775200` | `error` | `#BA1A1A` |
| `surface` | `#FBF8FF` | `on-surface` | `#1B1B21` |
| `surface-container-lowest` | `#FFFFFF` | `surface-container-low` | `#F5F2FA` |
| `surface-container` | `#EFEDF5` | `surface-container-high` | `#E9E7EF` |
| `surface-container-highest` | `#E3E1E9` | `surface-variant` | `#E2E1EF` |
| `on-surface-variant` | `#454651` | `outline` | `#767683` |
| `outline-variant` | `#C6C5D3` | `inverse-surface` | `#303036` |
| `inverse-on-surface` | `#F2EFF7` | `inverse-primary` | `#BAC3FF` |
| `primary-seed` | `#5C6BC0` | `surface-transparent-50` | `#FBF8FF80` |

字体族：HMCL 使用系统默认字体。Windows 回退链定为 `Microsoft YaHei UI` → `Segoe UI` → `sans-serif`，字号基准 12。

## 组件库（`ui/components/`）

逐项对应 HMCL 的构造组件，作为后续页面的唯一积木来源。

| HMCL 构造 | Slint 组件 | 说明 |
| --- | --- | --- |
| `RippleContainer` | `RippleArea` | Slint 无内建水波纹，用 `TouchArea` + 透明度动画近似；动效参数对齐 `SHORT4`/`EASE_IN` |
| `AdvancedListBox` / `AdvancedListItem` | `AdvancedListBox` / `AdvancedListItem` | 侧边导航，宽 200，条目内边距 `10 16`，标题 13、副标题 10 |
| `ClassTitle` | `ClassTitle` | 分组标题，12px，内边距 `8 16`，1px 分隔线 |
| `TwoLineListItem` | `TwoLineListItem` | 标题 15、副标题 12、标签圆角 2 |
| `ImageContainer` | `ImageContainer` | 默认圆角 6，支持 32/40/36 尺寸 |
| `LineButton` / `LineSelectButton` / `LineToggleButton` / `LineInheritableToggleButton` | `LineButton` 等 | 设置行基座，`MIN_HEIGHT 48`，间距 12，容器内边距 `10 16` |
| `LinePane` / `LineTextPane` | `LinePane` / `LineTextPane` | 左标题右编辑器的设置行 |
| `ComponentList` / `OptionsList` / `ComponentSublist` | `ComponentList` / `OptionsList` / `ComponentSublist` | 卡片分组，行内边距 `10 16`，首末圆角 4 |
| `Card` | `Card` | 圆角 4，`depth-1` 阴影，内边距 8 |
| `JFXDialogPane` / `JFXDialogLayout` | `DialogPane` | 圆角 4，内边距 `24 24 16 24`，标题 20 粗体 |
| `TabHeader` / `TabControl` / `TransitionPane` / `Navigator` | `TabHeader` / `Navigator` | 标签 16px、内边距 `10 17`；页面切换用 `FORWARD`/`BACKWARD` 动效 |
| `Decorator` / `MainWindowPane` | `WindowFrame` | 无边框窗口，内容圆角 8，四周 8px 阴影内边距，标题栏高 40 |
| `SpinnerPane` | `SpinnerPane` | 加载态 |
| `MemoryStatusBar` | `MemoryStatusBar` | 内存条，用于实例内存设置 |
| `PopupMenu` / `IconedMenuItem` | `PopupMenu` / `IconedMenuItem` | 右键菜单，容器内边距 `4 0`，条目 12px |
| Snackbar | `SnackBar` | 底部提示，背景 `inverse-surface` |

组件画廊：`app` 增加 `--ui-gallery` 调试入口，一屏渲染全部组件与状态，用于与 HMCL 截图并排校对。

## 页面映射

HMCL 页面较多，首版收敛到下列页面，保持 HMCL 的区域结构与版式，内容换为 Harness。

| HDSL 页面 | HMCL 对应 | 内容替换 |
| --- | --- | --- |
| 窗口外壳 | `RootPage` + `MainWindowPane` | 标题栏、返回/主页/刷新、侧边导航 |
| 首页 | `MainPage` | 启动面板、更新气泡、公告卡片 |
| 实例列表 | `GameListPage` | 工具栏（刷新/安装新版本/搜索）、实例单元格 |
| 实例详情 | `GameInstancePage` | 左侧三标签（实例设置 / 版本组件 / 插件管理）与底部工具条 |
| 下载 | `DownloadPage` | 左侧分类与右侧列表 |
| 版本选择 | `VersionsPage` | 版本列表、类型筛选、搜索 |
| 实例信息确认 | `InstallersPage` | 实例名、确认安装 |
| 设置 | `LauncherSettingsPage` | 8 个标签，见下 |
| 全局设置 | `GameSettingsPage<Preset>` | 默认实例设置 |
| 环境管理 | `JavaManagementPage` | Node.js / pnpm 运行时管理 |
| 通用 | `SettingsPage` | 更新、语言、杂项 |
| 外观 | `PersonalizationPage` | 主题模式、背景、动画、字体 |
| 下载 | `DownloadSettingsPage` | 下载源、并发、代理 |
| 帮助 / 反馈 / 关于 | `HelpPage` / `FeedbackPage` / `AboutPage` | 保留版式，链接改为 HDSL 资源 |
| 日志窗口 | `LogWindow` | 实例运行日志 |

设置侧边栏命名（按需求调整）：

| 分组 | 条目 | HMCL 原名 |
| --- | --- | --- |
| — | 全局设置 | 全局游戏设置（`settings.type.global.manage`） |
| — | 环境管理 | Java 管理（`java.management`） |
| 启动器 | 通用 | 通用（`settings.launcher.general`，不变） |
| 启动器 | 外观 | 外观（`settings.launcher.appearance`，不变） |
| 启动器 | 下载 | 下载（`download`，不变） |
| 帮助 | 帮助 | 帮助 |
| 帮助 | 反馈 | 反馈 |
| 帮助 | 关于 | 关于 |

### 已确认的映射

- 实例详情标签：只保留「实例设置 / 版本组件 / 插件管理」三个标签，HMCL 的模组管理、资源包管理、世界管理、原理图管理不实现。
- 首页侧边「账户」分组：显示当前用户的 GitHub 昵称与头像；账户登录与切换功能不实现，头像与昵称只读。
- 下载页分类：HMCL 的模组、资源包、光影、世界不适用，首版只保留「版本」与「插件」两个分类。

## 分阶段切片

每个切片按 [文档先行与 Git 工作流](workflow.md) 拆成 `docs(dev)` → `feat` → `docs(user)`，每个提交可构建。

| 阶段 | 内容 | 产出 | 状态 |
| --- | --- | --- | --- |
| S0 | 主题令牌、组件库、无边框窗口外壳、导航与页面切换 | `theme.slint`、`components/`、`app.slint` 外壳 | 已实现 |
| S1 | 首页 | `pages/home.slint` | 已实现 |
| S2 | 实例列表 | `pages/instances.slint` | 已实现 |
| S3 | 下载与安装 | `pages/install.slint` | 已实现 |
| S4 | 实例详情 | `pages/instance-detail.slint` | 已实现 |
| S5 | 设置（8 个标签） | `pages/settings.slint` | 已实现 |
| S6 | 对话框、提示、日志窗口 | `components/dialog.slint`、`pages/log.slint` | 已实现 |
| S7 | 动效、资源接入、像素校对 | 壁纸、页面淡入、刷新接线；逐项水波纹与像素校对待续 | 部分已实现 |

## 子智能体分工

- 每个阶段由一个子智能体独立实现，输入为该阶段的 HMCL 源码位置与本页的度量表。
- 子智能体必须先阅读根 `AGENTS.md` 与 `hdsl-rs/AGENTS.md`，只读 `reference/`，不得复制参考项目的源码、图标、壁纸或插画。
- 子智能体不得修改 `core` 的上游契约；UI 只通过现有回调与 `app` 交互。
- 每个子智能体在提交前运行 `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo test --workspace --locked`、两本 mdBook 构建与 `python scripts/check_docs.py`。
- 阶段之间只通过组件库接口耦合，避免并行冲突。

## 验收

- 与 HMCL 默认主题并排截图，区域布局、尺寸、间距、圆角、配色、字号一致；允许内容文字不同。
- `--ui-gallery` 能完整展示组件与状态，无渲染告警。
- 两条上游隔离约束仍成立：界面不提供写入 Harness 凭据或模型设置的入口；日志与错误中不出现密钥。
- CI 检查全绿：Rust 格式、clippy、测试、两本手册与文档链接检查。

## 风险

- Slint 与 JavaFX 能力差异：水波纹、模态弹窗定位、无边框窗口缩放热区、动效曲线需近似实现，需在 S0 先验证。
- Slint 无 CSS 级联，令牌与主题切换需用全局属性与组件参数表达。
- 工作量集中在页面数量；首版已收敛到约 12 页，Minecraft 专属页不在范围内。
- `@image-url` 不自动解析 `@2x` 变体，图形资源需按目标显示尺寸的两倍出图或使用 SVG，见 [UI 图形资源清单](ui-assets.md)。
