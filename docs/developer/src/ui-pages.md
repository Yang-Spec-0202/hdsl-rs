# UI 页面设计明细

状态：部分已实现。首页、实例列表、下载与安装、实例详情、设置、日志与对话框，以及全局外壳（壁纸、页面淡入、水波纹、窗口开合）均已按本页度量实现；设置项的持久化、主题与背景切换的实际生效留待后续。本页把 [UI 复刻计划：对齐 HMCL](ui-parity.md) 的页面映射展开为可实现的度量与验收条件。每个切片先在本页登记，再实现代码。

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

## 实例详情（`pages/instance-detail.slint`，对应 HMCL `GameInstancePage`）（已实现）

- 左侧标签栏宽 200，标签为「实例设置」「版本组件」「插件管理」，使用 `AdvancedListItem` 与图标；底部工具条含「测试游戏」「浏览」「管理」。
- 内容区随标签切换：实例设置 = 设置行列表（对应 HMCL 游戏设置）；版本组件 = 版本与组件列表；插件管理 = 插件列表。
- 页面接口：`in-out int page`、`in string instance-name`、`in string instance-version`、`in string instance-port`；数据由外壳从当前实例注入，首版标签内容用本地演示数据，持久化接线留待后续。
- 验收：三标签切换正常；左侧标签与底部工具条版式与 HMCL 一致；空实例时有占位提示。

## 设置（`pages/settings.slint`，对应 HMCL `LauncherSettingsPage`）（已实现）

- 左侧标签栏宽 200，按需求调整命名：`全局设置`（原「全局游戏设置」）、`环境管理`（原「Java 管理」）；分类「启动器」下为 `通用`、`外观`、`下载`（三者命名不变）；分类「帮助」下为 `帮助`、`反馈`、`关于`。
- 每个标签用 `ComponentList` + `LineComponent` 家族（`LineButton`/`LineSelectButton`/`LineToggleButton`/`LinePane`/`LineTextPane`）实现 HMCL 对应设置行。
- 首版只做界面与本地状态；设置项的持久化由后续切片接入。外观页的模型路由与凭据相关入口不提供（见 [架构与上游边界](architecture.md)）。
- 验收：八个标签齐全且命名正确；设置行版式（标题 14、副标题 11、行高 48/64、内边距 10/16）与 HMCL 一致；开关、下拉、输入框可用。

## 日志与对话框（`pages/log.slint`、`components/dialog.slint`）（已实现）

- 日志窗口对应 HMCL `LogWindow`：深色背景、等宽字体、按级别（trace/debug/info/warn/error/fatal）着色，顶部工具条含级别过滤与「始终置顶」开关。
- 对话框对应 HMCL `JFXDialogPane`：圆角 4、内边距 24/16、标题 20 粗体、操作按钮右对齐；补充底部 `SnackBar`（背景 `inverse-surface`，文字 `inverse-on-surface`，动作 `inverse-primary`）。
- 首版限制：Slint 无法在语言层按行切分字符串，日志正文以单块等宽文本呈现，尚未实现 HMCL 的逐行级别着色与过滤；级别按钮与「始终置顶」为界面占位。
- 验收：对话框与提示条的圆角、间距、配色与 HMCL 一致；日志窗口工具条版式与 HMCL 一致。

## 全局外壳（`app.slint`）（已实现）

- 无边框窗口：内容圆角 8，四周 8px 阴影内边距；标题栏高 40，含返回、主页、标题、帮助、最小化、关闭。
- 背景：内容区用 `wallpapers/roxy-bot-light-gradient.jpg` 铺满（`image-fit: cover`），侧栏与内容区半透明以透出壁纸，对应 HMCL `MainWindowPane` 的背景节点。
- 页面切换：各页面常驻并按 `page` 切换可见性，`opacity` 以 200ms `ease-in-out` 淡入，近似 HMCL 的 `FADE`/`FORWARD` 过渡。
- 水波纹：`RippleArea` 提供 4% 悬停遮罩，以及按下时从触点扩散、松开时淡出的圆形水波纹（`transform-scale-*`，350ms `ease-out`）；侧边项与设置行复用。
- 窗口开合：启动时内容以 400ms 从 0.8 缩放并淡入；关闭时先 220ms 淡出缩小，再由定时器退出，对应 HMCL `Decorator` 的窗口动画。
- 数据接线：首页更新气泡由 `latest-version` 驱动；实例列表「刷新」调用 `refresh-instances`。
- 验收：切换页面有淡入动效；按下侧边项与设置行有扩散水波纹；窗口淡入淡出正常；壁纸铺满且不遮挡文字；窗口可拖动、可最小化与关闭。

## 待实现切片

实例设置与设置的持久化接线、插件市场与下载列表的数据接线、主题与背景切换的实际生效、实例图标选择对话框、日志的逐行级别着色与过滤、HMCL 的逐项水波纹与窗口开合动效，留待后续切片。
