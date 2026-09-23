# 页面规范

各页面的尺寸与交互定义如下。实现程度及限制见[功能状态与限制](status.md)。本页不记录功能进度。

令牌名对应 `ui/theme.slint` 的 `Theme`，复用组件位于 `ui/components/`。

## 首页（`pages/home.slint`）

- 页面内边距 20；内容右下角为启动面板，右上角为更新气泡，顶部为公告卡片。
- 启动面板：整体 230×57，右下对齐；启动按钮 200×55，圆角 `4 0 0 4`，右侧 3px 分隔条；菜单按钮 27×55，圆角 `0 4 4 4`，图标 `arrow-drop-up` 30px；背景 `primary-container`，文字 `on-primary-container`。
- 启动按钮文案：无实例时为「开始游戏」，有实例时为「启动游戏」并显示实例标识副行（12px）；主文案 16px。
- 更新气泡：230×55，右上对齐；背景 `inverse-surface-transparent-80`，圆角 2；图标 `update` 20px；标题「发现更新：%s」，副标题「点击此处进行升级」；关闭按钮 15px。无更新时隐藏。
- 公告卡片：内边距 16，间距 16；背景 `surface-container-low-transparent-80`，圆角 4，带阴影；标题 14 粗体，正文 13；关闭按钮 20×20。内容为上游版本公告。

## 实例列表（`pages/instances.slint`）

- 工具栏：按钮高 37、圆角 5，依次为 刷新（`refresh`）、安装新版本（`download`）、搜索（`search`）；点击搜索切换为搜索栏（输入框 + 关闭按钮），搜索框提示「搜索」。
- 列表单元格：底部 1px 分隔线 `outline-variant`；左侧 32px 实例图标（默认 `instances/grass.png`）；中部两行文本（标题 15 = 实例名，副标题 12 = 版本 + 工作目录）；右侧操作按钮 30×30（启动 `rocket-launch`、管理 `more-vert`）。
- 选中行背景 `secondary-container`；空列表提示「没有已安装的游戏。」。
- 页面接口保持 `instances`、`selected-id`、`pending-delete-id` 与 `select-instance`、`launch-instance`、`clone-instance` 回调不变。

## 下载与安装（`pages/install.slint`）

- 安装页提供版本筛选与创建实例表单；插件入口位于独立的插件市场页面。
- 版本页顶部搜索卡片：内边距 10，`hgap` 16；「名称」输入框（提示「输入版本名称进行搜索」）+「版本类型」下拉（全部/正式版/预览版）+ 刷新按钮。
- 版本列表单元格：32px 图标 + 两行文本（标题 = 版本号，副标题 = 发布日期）+ 标签（正式版/预览版）；整行可点击选中。
- 安装确认：实例名输入框（提示「游戏实例名称」）+ 底部「安装」按钮（100×40，右对齐）。
- 页面接口保持 `form-name`、`form-version`、`form-workspace`、`available-versions`、`available-version-count` 与 `refresh-versions`、`create-instance` 回调不变。

## 实例详情（`pages/instance-detail.slint`）

- 左侧标签栏宽 200，标签为「实例设置」「版本组件」「插件管理」，使用 `AdvancedListItem` 与图标；底部工具条含「测试游戏」「浏览」「管理」。
- 内容区随标签切换：实例设置 = 设置行列表（实例设置）；版本组件 = 版本与组件列表；插件管理 = 插件列表。
- 页面接口：`in-out int page`、`in string instance-name`、`in string instance-version`、`in string instance-port`；数据由外壳从当前实例注入。

## 设置（`pages/settings.slint`）

- 左侧标签栏宽 200，命名为：`全局设置`（原「全局游戏设置」）、`环境管理`（原「Java 管理」）；分类「启动器」下为 `通用`、`外观`、`下载`（三者命名不变）；分类「帮助」下为 `帮助`、`反馈`、`关于`。
- 每个标签用 `ComponentList` + `LineComponent` 家族（`LineButton`/`LineSelectButton`/`LineToggleButton`/`LinePane`/`LineTextPane`）实现设置行。
- 设置项的实际生效范围见[功能状态与限制](status.md)。外观页的模型路由与凭据相关入口不提供（见 [架构与上游边界](architecture.md)）。

## 日志与对话框（`pages/log.slint`、`components/dialog.slint`）

- 日志窗口参考 `LogWindow`：深色背景、等宽字体、按级别（trace/debug/info/warn/error/fatal）逐行着色，顶部工具条含级别过滤与「始终置顶」开关。
- 对话框参考 `JFXDialogPane`：圆角 4、内边距 24/16、标题 20 粗体、操作按钮右对齐；补充底部 `SnackBar`（背景 `inverse-surface`，文字 `inverse-on-surface`，动作 `inverse-primary`）。
- 日志由应用层按行送入页面，按级别着色和筛选；“始终置顶”尚未连接系统窗口。

## 全局外壳（`app.slint`）

- 无边框窗口：内容圆角 8，四周 8px 阴影内边距；标题栏高 40，含返回、主页、标题、帮助、最小化、关闭。
- 背景：内容区用 `wallpapers/roxy-bot-light-gradient.jpg` 铺满（`image-fit: cover`），侧栏与内容区半透明，以显示背景。
- 页面切换：各页面常驻并按 `page` 切换可见性，`opacity` 以 200ms `ease-in-out` 淡入，提供页面切换过渡。
- 水波纹：`RippleArea` 提供 4% 悬停遮罩，以及按下时从触点扩散、松开时淡出的圆形水波纹（`transform-scale-*`，350ms `ease-out`）；侧边项与设置行复用。
- 窗口开合：启动时内容以 400ms 从 0.8 缩放并淡入；关闭时先 220ms 淡出缩小，再由定时器退出。
- 数据接线：首页更新气泡由 `latest-version` 驱动；实例列表「刷新」调用 `refresh-instances`。
