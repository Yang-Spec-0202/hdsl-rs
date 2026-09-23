# UI 页面设计明细

状态：部分已实现。首页、实例列表、下载与安装页已按本页度量实现；实例详情与设置页待实现。本页把 [UI 复刻计划：对齐 HMCL](ui-parity.md) 的页面映射展开为可实现的度量与验收条件。每个切片先在本页登记，再实现代码。

约定：尺寸与颜色来自 HMCL 默认主题（`blue.css`、`root.css`），令牌名对应 `ui/theme.slint` 的 `Theme`。页面只消费 `ui/components/` 的组件；页面专用控件先写在页面文件内，稳定后再提升为共享组件。

## 首页（`pages/home.slint`，对应 HMCL `MainPage`）（已实现）

- 页面内边距 20；内容右下角为启动面板，右上角为更新气泡，顶部为公告卡片。
- 启动面板：整体 230×57，右下对齐；启动按钮 200×55，圆角 `4 0 0 4`，右侧 3px 分隔条；菜单按钮 27×55，圆角 `0 4 4 4`，图标 `arrow-drop-up` 30px；背景 `primary-container`，文字 `on-primary-container`。
- 启动按钮文案：无实例时为「开始游戏」，有实例时为「启动游戏」并显示实例标识副行（12px）；主文案 16px。
- 更新气泡：230×55，右上对齐；背景 `inverse-surface-transparent-80`，圆角 2；图标 `update` 20px；标题「发现更新：%s」，副标题「点击此处进行升级」；关闭按钮 15px。无更新时隐藏。
- 公告卡片：内边距 16，间距 16；背景 `surface-container-low-transparent-80`，圆角 4，带阴影；标题 14 粗体，正文 13；关闭按钮 20×20。内容为上游版本公告。
- 验收：三种元素位置与 HMCL 一致；无实例、有实例、有更新、无更新四种状态都能正确显示；点击启动/停止调用既有回调。

## 实例列表（`pages/instances.slint`，对应 HMCL `GameListPage`）（已实现）

- 工具栏：按钮高 37、圆角 5，依次为 刷新（`refresh`）、安装新版本（`download`）、搜索（`search`）；点击搜索切换为搜索栏（输入框 + 关闭按钮），搜索框提示「搜索」。
- 列表单元格：底部 1px 分隔线 `outline-variant`；左侧 32px 实例图标（默认 `instances/grass.png`）；中部两行文本（标题 15 = 实例名，副标题 12 = 版本 + 工作目录）；右侧操作按钮 30×30（启动 `rocket-launch`、管理 `more-vert`）。
- 选中行背景 `secondary-container`；空列表提示「没有已安装的游戏。」。
- 页面接口保持 `instances`、`selected-id`、`pending-delete-id` 与 `select-instance`、`launch-instance`、`clone-instance` 回调不变。
- 验收：单元格版式与 HMCL `GameListCell` 一致；空态、多实例、选中态正确；启动/删除走既有回调。

## 下载与安装（`pages/install.slint`，对应 HMCL `DownloadPage` + `VersionsPage` + `InstallersPage`）（已实现）

- 左侧分类栏宽 200：分类「新游戏」下的「版本」，分类「游戏内容」下的「插件」（Harness 无模组/资源包/光影/世界，首版只保留版本与插件）。
- 版本页顶部搜索卡片：内边距 10，`hgap` 16；「名称」输入框（提示「输入版本名称进行搜索」）+「版本类型」下拉（全部/正式版/预览版）+ 刷新按钮。
- 版本列表单元格：32px 图标 + 两行文本（标题 = 版本号，副标题 = 发布日期）+ 标签（正式版/预览版）；整行可点击选中。
- 安装确认：顶部实例名输入框（提示「游戏实例名称」）+ 底部「安装」按钮（100×40，右对齐）。
- 页面接口保持 `form-name`、`form-version`、`form-workspace`、`available-versions`、`available-version-count` 与 `refresh-versions`、`create-instance` 回调不变。
- 验收：版本列表可滚动、可筛选、可选中；预览版有明确标记；创建实例走既有回调。

## 待实现切片

实例详情（对应 `GameInstancePage`，标签「实例设置/版本组件/插件管理」）、设置（对应 `LauncherSettingsPage`，标签「全局设置/环境管理/通用/外观/下载/帮助/反馈/关于」）、对话框与日志窗口的度量在各自切片开始前补入本页。
