# UI 图形资源清单

资源规格和引用路径记录在本页；当前替换进度见[功能状态与限制](status.md)。图形资源由本项目独立创作，不能复制参考项目资产。

## 约定

- 目录：`crates/ui/ui/assets/`，按 `account/`、`branding/`、`wallpapers/`、`instances/`、`placeholders/`、`social/`、`icons/` 分子目录。
- 仓库在干净检出后必须可构建，因此每个被引用的路径都有一份原创占位图（提交内容）。见下文「开发期临时资源」。
- Slint 的 `@image-url` 不自动解析 `@2x` 变体，也不按扩展名推断格式。位图按目标显示尺寸的两倍出图，文件名带 `@2x` 后缀并由组件显式引用；矢量图标用 SVG。
- 单色 UI 图标统一 24×24 视框、`fill="#000000"`，由 `Icon` 组件用 `colorize` 着色。
- 位图使用透明背景 PNG；壁纸使用 JPEG。

## 资源使用

仓库提交可构建的原创资源。图标引用路径与 `crates/ui/ui/icons.slint` 保持一致；替换占位图时核对不同分辨率的显示效果。开发时的临时资源使用规则见根目录 `AGENTS.md`。

## A. 应用标识（branding）

已并入原创 Roxy Bot 标识。

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `branding/icon.svg` | 矢量（1024 视框） | 主标识（关于页、默认实例图标） |
| `branding/icon.png` | 32×32 | 主标识位图 |
| `branding/icon@2x.png` | 64×64 | HiDPI 标识 |
| `branding/icon@4x.png` | 128×128 | Linux 窗口图标 |
| `branding/icon@8x.png` | 256×256 | 关于页/主题大图标 |
| `branding/icon-mac.png` | 512×512 | macOS Dock 图标 |
| `branding/icon-title.svg` | 矢量 | 标题栏图标 |
| `branding/icon-title@2x.png` | 48×48 | 标题栏 HiDPI |
| `branding/roxy-bot-mascot-cutout.png` | 1254×1254 | Roxy Bot 角色立绘（首页/关于页备用） |

现有 `assets/hdsl-mark.svg` 保留备用。

## B. 内置壁纸（wallpapers）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `wallpapers/roxy-bot-light-gradient.jpg` | 1600×900 | 默认背景 |

## C. 实例图标集（instances）

实例图标现有部分历史命名；Harness 无模组加载器，这些图标槽位（`fabric`、`forge`、`neoforge`、`quilt`、`legacyfabric`、`cleanroom`、`optifine`）在原创资源定稿时改为 Harness 语义的通用图标，并同步更新引用。统一像素风，每项出 32×32 与 64×64（`@2x`）。

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

单色图标的引用和文件名以 `crates/ui/ui/icons.slint` 为准。仓库内部分图标仍为原创占位图。分组示例：

| 分组 | 文件名示例 |
| --- | --- |
| 窗口与导航 | `arrow-back` `home` `refresh` `close` `minimize-center` `help` `help-fill` |
| 侧边导航 | `list-bulleted` `download` `settings` `settings-fill` `graph2` `chat` `add-circle` `folder` `folder-fill` `controller` `package2` `extension` `texture` `wb-sunny` `public` `deployed-code` `schema` `local-cafe` `tune` `style` `feedback` `info` |
| 首页与列表 | `arrow-drop-up` `update` `rocket-launch` `more-vert` `more-horiz` `menu` |
| 列表与操作 | `add` `edit` `delete` `delete-forever` `folder-open` `folder-copy` `output` `script` `search` `unfold-more` `arrow-forward` `check` `cancel` `select-all` `restore` `globe-book` `alpha-circle` `beta-circle` `release-circle` `visibility` `visibility-off` `archive` `keyboard-arrow-down` `keyboard-arrow-up` `content-copy` `warning` `error` `person` `host` `explore` `fort` `location-city` `screenshot-monitor` `frame-bug` |

## G. 账户头像（account）

| 文件 | 尺寸 | 用途 |
| --- | --- | --- |
| `account/avatar.jpg` | 64×64 以上 | 首页「环境」分组的用户头像（当前为 GitHub 头像，只读显示） |
