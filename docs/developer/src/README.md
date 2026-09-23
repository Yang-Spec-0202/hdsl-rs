# HDSL 开发者手册

功能完成度统一记录在[功能状态与限制](status.md)。本手册描述当前架构、设计约束和开发流程。

首版目标是在 Windows x64、Linux x64 和 Linux arm64 上，以独立运行时安装和启动指定版本的 DeepSeek Harness。图形界面负责实例与经过版本核验的插件；模型路由与凭据由实例的 Harness 本体管理。主要运行面为官方 `web` profile。

不导入 Java HDSL 的旧数据，也不实现 Harness 的 agent 本体。工程位于独立 Git 仓库，参考目录不纳入版本控制。

