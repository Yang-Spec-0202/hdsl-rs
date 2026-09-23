# 项目范围与路线

HDSL 为 Windows 和 Linux 提供 DeepSeek Harness 桌面启动体验。实例拥有各自的 Harness 版本、工作目录、端口和 `DSH_HOME`；插件操作使用当前实例的官方 `dsh plugin --profile web`。模型路由与凭据由 Harness 本体维护。

当前实现和限制记录在[功能状态与限制](status.md)。工程边界见[架构与上游边界](architecture.md)，插件准入规则见[插件准入与兼容性](compatibility.md)，界面规范见[界面设计](ui-parity.md)和[页面规范](ui-pages.md)。

## 后续工作

- 完成设置持久化、外观应用与实例详情中尚未接入的操作。
- 改进插件操作进度及错误提示，完成跨 Harness 版本的真实插件测试。
- 将占位图替换为正式原创资源，完成目标平台打包与发行验证。

发行前的检查项目见[构建、测试与发行](release.md)。HDSL 不导入 Java 版数据，也不实现 Harness 的 agent 本体或 Minecraft 专属管理功能。
