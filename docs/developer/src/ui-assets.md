# UI 图形资源清单

状态：计划中。本页列出 [UI 复刻计划：对齐 HMCL](ui-parity.md) 需要原创生成的图形资源。所有资源必须原创，不得复制 HMCL、HMCL-rs 或 DeepSeek 娘的任何图标、壁纸或插画（见根 `AGENTS.md`）。

## 约定

- 目录：`crates/ui/ui/assets/`，按 `branding/`、`wallpapers/`、`icons/`、`instances/`、`placeholders/`、`social/` 分子目录。
- Slint 的 `@image-url` 不自动解析 `@2x` 变体。图标优先出 SVG；位图按目标显示尺寸的两倍出图，并在文件名带 `@2x` 后缀，由组件显式引用。
- 单色 UI 图标统一 24×24 视框、`fill="currentColor"`，由组件用 `colorize` 着色。
- 位图使用透明背景 PNG；壁纸用 JPEG。

## 开发期临时资源

仓库在干净检出后必须可构建，因此 `crates/ui/ui/assets/` 中始终存在原创占位图。开发期用 HMCL 参考资源覆盖同名占位图来校对版式：

- `scripts/make-placeholder-assets.ps1` 生成原创占位图（提交内容）。
- `scripts/stage-reference-assets.ps1` 用 `reference/HMCL` 的位图与从 `SVG.java` 提取的矢量图标覆盖同名占位图，仅供本地开发。
- 覆盖后的文件是参考资源，**提交前必须还原**：运行 `scripts/make-placeholder-assets.ps1`，或 `git checkout -- crates/ui/ui/assets`。
- 原创资源就绪后，直接替换 `crates/ui/ui/assets/` 下同名文件并提交。

## A. 应用标识（branding）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `branding/icon.svg` | 矢量 | 主标识（窗口、关于页、默认实例图标） |
| `branding/icon@2x.png` | 64×64 | HiDPI 标识 |
| `branding/icon@4x.png` | 128×128 | Linux 窗口图标 |
| `branding/icon@8x.png` | 256×256 | 关于页/主题大图标 |
| `branding/icon-mac.png` | 512×512 | macOS Dock 图标 |
| `branding/icon-title.svg` | 24×24 | 标题栏图标 |
| `branding/icon-title@2x.png` | 48×48 | 标题栏 HiDPI |

现有 `assets/hdsl-mark.svg` 可复用或替换。

## B. 内置壁纸（wallpapers）

与 HMCL 同 id、同尺寸，内容原创。

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `wallpapers/2021-08-26.jpg` | 1600×900 | 默认背景 |
| `wallpapers/2016-02-25.jpg` | 800×480 | 备选背景 |
| `wallpapers/2015-06-22.jpg` | 800×480 | 备选背景 |

## C. 实例图标集（instances）

HMCL 的实例图标对话框为「自定义 + 14 个内置」共 15 槽。Harness 无模组加载器，加载器图标槽改为通用装饰图标。统一像素风，每项出 32×32 与 64×64（`@2x`）。

| 文件名 | 尺寸 | 用途 |
| --- | --- | --- |
| `instances/add.svg` | 24×24 | 图标对话框「自定义」按钮 |
| `instances/default.png` (+`@2x`) | 32×32 / 64×64 | 默认实例图标 |
| `instances/cube.png` (+`@2x`) | 32×32 / 64×64 | 备选 1 |
| `instances/terminal.png` (+`@2x`) | 32×32 / 64×64 | 备选 2 |
| `instances/rocket.png` (+`@2x`) | 32×32 / 64×64 | 备选 3 |
| `instances/gear.png` (+`@2x`) | 32×32 / 64×64 | 备选 4 |
| `instances/flask.png` (+`@2x`) | 32×32 / 64×64 | 备选 5 |
| `instances/package.png` (+`@2x`) | 32×32 / 64×64 | 备选 6 |
| `instances/cloud.png` (+`@2x`) | 32×32 / 64×64 | 备选 7 |
| `instances/chip.png` (+`@2x`) | 32×32 / 64×64 | 备选 8 |
| `instances/star.png` (+`@2x`) | 32×32 / 64×64 | 备选 9 |
| `instances/moon.png` (+`@2x`) | 32×32 / 64×64 | 备选 10 |
| `instances/fox.png` (+`@2x`) | 32×32 / 64×64 | 备选 11 |
| `instances/whale.png` (+`@2x`) | 32×32 / 64×64 | 备选 12 |
| `instances/spark.png` (+`@2x`) | 32×32 / 64×64 | 备选 13 |
| `instances/leaf.png` (+`@2x`) | 32×32 / 64×64 | 备选 14 |

## D. 占位图（placeholders）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `placeholders/unknown-addon.png` | 128×128 | 插件/扩展列表默认图 |
| `placeholders/unknown-workspace.png` | 128×128 | 工作区/会话列表默认图 |

## E. 社交与社区（social）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `social/github.svg` | 24×24 | 关于页/反馈页，浅色 |
| `social/github-white.svg` | 24×24 | 深色背景 |
| `social/discord.svg` | 24×24 | 社区入口 |
| `social/community.svg` | 24×24 | 官方群组入口 |

## F. 单色 UI 图标（icons，24×24 SVG）

按 HMCL 的 SVG 枚举逐项对齐。全部 24×24 视框、单色。

| 分组 | 文件名 |
| --- | --- |
| 窗口与导航 | `arrow-back` `home` `refresh` `close` `minimize-center` `help` `help-fill` |
| 侧边导航 | `list-bulleted` `download` `settings` `settings-fill` `graph2` `chat` `add-circle` `folder` `folder-fill` `controller` `controller-fill` `package2` `package2-fill` `extension` `extension-fill` `texture` `wb-sunny` `wb-sunny-fill` `public` `deployed-code` `deployed-code-fill` `schema` `schema-fill` `local-cafe` `local-cafe-fill` `tune` `style` `style-fill` `feedback` `feedback-fill` `info` `info-fill` |
| 首页 | `arrow-drop-up` `update` `rocket-launch` `more-vert` `more-horiz` `menu` |
| 列表与操作 | `add` `edit` `delete` `delete-forever` `folder-open` `folder-copy` `output` `script` `search` `unfold-more` `arrow-forward` `check` `cancel` `select-all` `restore` `globe-book` `alpha-circle` `beta-circle` `release-circle` `visibility` `visibility-off` `archive` `keyboard-arrow-down` `keyboard-arrow-up` `content-copy` `warning` `error` `person` `host` `explore` `fort` `location-city` `screenshot-monitor` `frame-bug` |

## G. 关于页头像（可选）

关于页若复刻 HMCL 的贡献者列表，需要 HDSL 贡献者头像，每人 32×32 与 64×64，命名与 `thanks` 数据一致。此项由 HDSL 贡献者名单决定，暂不列出具体文件。

## 生成优先级

1. 先出 F 组单色图标与 A 组标识，S0 组件库与外壳依赖它们。
2. 再出 C 组实例图标与 D 组占位图，S2、S3 依赖。
3. B 组壁纸与 E 组社交图标在 S5 外观页前完成。
