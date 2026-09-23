# UI 图形资源清单

状态：部分已实现。本页列出 [UI 复刻计划：对齐 HMCL](ui-parity.md) 使用与需要的图形资源。所有资源必须原创，不得复制 HMCL、HMCL-rs 或 DeepSeek 娘的任何图标、壁纸或插画（见根 `AGENTS.md`）。

## 约定

- 目录：`crates/ui/ui/assets/`，按 `account/`、`branding/`、`wallpapers/`、`instances/`、`placeholders/`、`social/`、`icons/` 分子目录。
- 仓库在干净检出后必须可构建，因此每个被引用的路径都有一份原创占位图（提交内容）。见下文「开发期临时资源」。
- Slint 的 `@image-url` 不自动解析 `@2x` 变体，也不按扩展名推断格式。位图按目标显示尺寸的两倍出图，文件名带 `@2x` 后缀并由组件显式引用；矢量图标用 SVG。
- 单色 UI 图标统一 24×24 视框、`fill="#000000"`，由 `Icon` 组件用 `colorize` 着色。
- 位图使用透明背景 PNG；壁纸使用 JPEG。

## 开发期临时资源

开发期用 HMCL 参考资源覆盖同名占位图来校对版式：

- `scripts/make-placeholder-assets.ps1` 生成原创占位图（提交内容）。
- `scripts/stage-reference-assets.ps1` 用 `reference/HMCL` 的位图与从 `SVG.java` 提取的矢量图标覆盖同名占位图，仅供本地开发。
- 覆盖后的文件是参考资源，**提交前必须还原**：运行 `scripts/make-placeholder-assets.ps1`，或 `git checkout -- crates/ui/ui/assets`。
- 原创资源就绪后，直接替换 `crates/ui/ui/assets/` 下同名文件并提交。

## A. 应用标识（branding）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `branding/icon.png` | 32×32 | 主标识（关于页、默认实例图标） |
| `branding/icon@2x.png` | 64×64 | HiDPI 标识 |
| `branding/icon@4x.png` | 128×128 | Linux 窗口图标 |
| `branding/icon@8x.png` | 256×256 | 关于页/主题大图标 |
| `branding/icon-mac.png` | 512×512 | macOS Dock 图标 |
| `branding/icon-title.png` | 24×24 | 标题栏图标 |
| `branding/icon-title@2x.png` | 48×48 | 标题栏 HiDPI |

现有 `assets/hdsl-mark.svg` 保留备用。

## B. 内置壁纸（wallpapers）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `wallpapers/2021-08-26.jpg` | 1600×900 | 默认背景 |
| `wallpapers/2016-02-25.jpg` | 800×480 | 备选背景 |
| `wallpapers/2015-06-22.jpg` | 800×480 | 备选背景 |

## C. 实例图标集（instances）

首版沿用 HMCL 的文件名，便于用参考资源同名替换；Harness 无模组加载器，加载器槽位（`fabric`、`forge`、`neoforge`、`quilt`、`legacyfabric`、`cleanroom`、`optifine`）在原创资源定稿时改为 Harness 语义的通用图标，并同步更新引用。统一像素风，每项出 32×32 与 64×64（`@2x`）。

| 文件（基名） | 尺寸 | 用途 |
| --- | --- | --- |
| `instances/grass` | 32×32 / 64×64 | 默认实例图标 |
| `instances/chest` | 32×32 / 64×64 | 备选 |
| `instances/chicken` | 32×32 / 64×64 | 备选 |
| `instances/command` | 32×32 / 64×64 | 备选 |
| `instances/april_fools` | 32×32 / 64×64 | 备选 |
| `instances/optifine` | 32×32 / 64×64 | 备选 |
| `instances/craft_table` | 32×32 / 64×64 | 备选 |
| `instances/fabric` | 32×32 / 64×64 | 备选 |
| `instances/legacyfabric` | 32×32 / 64×64 | 备选 |
| `instances/forge` | 32×32 / 64×64 | 备选 |
| `instances/cleanroom` | 32×32 / 64×64 | 备选 |
| `instances/neoforge` | 32×32 / 64×64 | 备选 |
| `instances/furnace` | 32×32 / 64×64 | 备选 |
| `instances/quilt` | 32×32 / 64×64 | 备选 |

## D. 占位图（placeholders）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `placeholders/unknown_pack.png` | 128×128 | 资源/扩展列表默认图 |
| `placeholders/unknown_server.png` | 128×128 | 工作区/会话列表默认图 |

## E. 社交与社区（social）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `social/github.png` | 32×32 | 关于页/反馈页 |
| `social/github-white.png` | 32×32 | 深色背景 |
| `social/discord.png` | 32×32 | 社区入口 |

## F. 单色 UI 图标（icons）

图标集与 HMCL `SVG.java` 枚举一一对应，共 100 个，文件名为枚举名小写、下划线转连字符（如 `ADD_CIRCLE` → `add-circle.svg`）。完整清单见 `crates/ui/ui/icons.slint`，由 `scripts/gen-icons.ps1` 生成。分组示例：

| 分组 | 文件名示例 |
| --- | --- |
| 窗口与导航 | `arrow-back` `home` `refresh` `close` `minimize-center` `help` `help-fill` |
| 侧边导航 | `list-bulleted` `download` `settings` `settings-fill` `graph2` `chat` `add-circle` `folder` `folder-fill` `stadia-controller` `package2` `extension` `texture` `wb-sunny` `public` `deployed-code` `schema` `local-cafe` `tune` `style` `feedback` `info` |
| 首页与列表 | `arrow-drop-up` `update` `rocket-launch` `more-vert` `more-horiz` `menu` |
| 列表与操作 | `add` `edit` `delete` `delete-forever` `folder-open` `folder-copy` `output` `script` `search` `unfold-more` `arrow-forward` `check` `cancel` `select-all` `restore` `globe-book` `alpha-circle` `beta-circle` `release-circle` `visibility` `visibility-off` `archive` `keyboard-arrow-down` `keyboard-arrow-up` `content-copy` `warning` `error` `person` `host` `explore` `fort` `location-city` `screenshot-monitor` `frame-bug` |

## G. 账户头像（account）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `account/avatar.jpg` | 64×64 以上 | 首页「环境」分组的用户头像（当前为 GitHub 头像，只读显示） |

## 生成优先级

1. F 组单色图标与 A 组标识，组件库与外壳依赖它们。
2. C 组实例图标与 D 组占位图。
3. B 组壁纸与 E 组社交图标。
4. G 组账户头像在用户身份确定后替换。
