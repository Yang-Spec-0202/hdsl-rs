# 界面设计

实现范围与限制见[功能状态与限制](status.md)；各页面的尺寸与交互见[页面规范](ui-pages.md)，图形资源见[UI 图形资源清单](ui-assets.md)。

## 设计范围

设计目标：

- 导航层级与交互方式参考 HMCL；实际视觉参数以仓库中的 Slint 组件和令牌为准。
- HDSL 内容为 DeepSeek Harness：实例 = Harness 实例，版本 = Harness 版本，插件 = 兼容插件。
- 图形资源全部原创，规格见 [UI 图形资源清单](ui-assets.md)。

不在当前设计范围内：

- 不实现参考项目的 Minecraft 专属功能：模组、资源包、世界、原理图、NBT 编辑器、多人联机、账户与皮肤。
- 不实现主题包市场、主题色生成器、字体选择等外观高级项；外观页的交互范围见[功能状态与限制](status.md)。

## 设计令牌

`ui/theme.slint` 提供颜色、字号、间距及动效令牌，供组件统一读取。配色参考 Material 3 的角色划分。

| 类别 | 取值 |
| --- | --- |
| 颜色角色 | 50 个 `-monet-*` 角色 + `primary-seed` + 9 个透明变体，见下表 |
| 字号 | 10 / 12 / 13 / 14 / 15 / 16 / 18 / 20 / 22 |
| 圆角 | 2 / 3 / 4 / 5 / 6 / 20 / 25 / 50 |
| 间距 | 0 / 2 / 4 / 5 / 6 / 8 / 10 / 12 / 16 / 22 / 24 |
| 阴影 | `depth-0`…`depth-5`，`card`、`options-list`、`card-non-transparent` |
| 动效 | `SHORT1..4`=50/100/150/200ms；`MEDIUM1..4`=250/300/350/400ms；`LONG1..4`=450/500/550/600ms；曲线 `EASE`/`EASE_IN`/`EASE_OUT`/`EASE_IN_OUT`/`STANDARD`/`EMPHASIZED_*` |

当前主题关键色：

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

字体族采用系统默认字体。Windows 回退链定为 `Microsoft YaHei UI` → `Segoe UI` → `sans-serif`，字号基准 12。

## 组件与页面

`ui/components/` 提供导航、设置行、列表、卡片、对话框和提示等共享组件。`ui/pages/` 组合这些组件，`ui/app.slint` 管理导航与事件连接。页面专用的交互保留在对应页面中，以免共享组件承担不相关的状态。

| 区域 | 内容与交互 |
| --- | --- |
| 首页 | 实例启动入口、更新提示、上游公告 |
| 实例列表和详情 | 搜索、选择、运行与删除；详情提供实例信息和扩展入口 |
| 下载与安装 | Harness 版本查询、筛选与实例创建；插件入口导向插件市场 |
| 插件市场 | 根据当前实例筛选兼容版本，执行安装与卸载 |
| 设置 | 全局设置、环境管理、通用、外观、下载、帮助、反馈、关于 |
| 日志 | 运行输出、级别筛选与状态提示 |

设置侧栏的“环境管理”面向 Node.js 和 pnpm，而不是 Java；与 Harness 的模型路由和凭据有关的操作仍由 Harness 自身提供。页面布局和控件尺寸见[页面规范](ui-pages.md)。

## 交互与验证

页面通过 `app.slint` 的属性和回调连接应用服务。复用组件放在 `ui/components/`，页面专属控件保留在对应页面目录。界面操作应能在空实例、安装中、失败和多实例状态下给出明确反馈；没有接入应用服务的控件需在[功能状态与限制](status.md)中标明。

视觉检查包括常见窗口尺寸下的布局、文本可读性、动效以及无边框窗口的拖动与缩放。上游隔离约束与日志脱敏按[架构与上游边界](architecture.md)核对。

## 风险

- Slint 与 JavaFX 能力差异：水波纹、模态弹窗定位、无边框窗口缩放热区、动效曲线需近似实现，需通过实际界面检查。
- Slint 无 CSS 级联，令牌与主题切换需用全局属性与组件参数表达。
- 页面较多，新增共享组件时应核对所有使用它的页面。
- `@image-url` 不自动解析 `@2x` 变体，图形资源需按目标显示尺寸的两倍出图或使用 SVG，见 [UI 图形资源清单](ui-assets.md)。
