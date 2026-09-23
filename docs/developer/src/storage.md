# 实例与磁盘布局

状态：已实现。实例按 UUID 保存，精确版本与工作目录在创建时验证。

安装版的数据根目录为 Windows `%LOCALAPPDATA%/hdsl-rs` 或 Linux `${XDG_DATA_HOME:-~/.local/share}/hdsl-rs`。便携版在可执行文件旁有 `portable.flag`，数据根为同级 `data/`。两种模式不会自动读取 Java HDSL 数据。

根目录包含 `runtimes/node/<version>/`、`tools/pnpm/<version>/`、`instances/<id>/`、`catalog/` 和 `settings.json`。每个实例含 `instance.json`、`dsh/`、`home/`；`dsh/` 持有精确版本，`home/` 是唯一的 `DSH_HOME`。实例 ID 是生成的 UUID，不从用户名称构造路径。

升级创建新实例和新 home；原实例保留。删除只移除启动器拥有的实例目录。写入 JSON 使用临时文件与同目录替换；配置含 schema 版本，未知较新 schema 只读并提示升级启动器。

验收：崩溃或断电后不出现半写入的实例清单；旧版本实例在升级后仍可启动。
