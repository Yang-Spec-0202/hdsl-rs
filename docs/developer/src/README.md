# HDSL 开发者手册

状态：部分已实现。各页在实现前确定边界与验收条件，并随代码提交更新为“已实现”或“部分已实现”。

首版目标是在 Windows x64、Linux x64 和 Linux arm64 上，以独立运行时安装和启动指定版本的 DeepSeek Harness。图形界面负责实例与经过版本核验的插件；模型路由与凭据由实例的 Harness 本体管理。主要运行面为官方 `web` profile。

不导入 Java HDSL 的旧数据，也不实现 Harness 的 agent 本体。工程位于独立 Git 仓库，参考目录不纳入版本控制。

