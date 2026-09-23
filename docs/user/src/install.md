# 安装与启动

当前开发预览版可从仓库构建。需要 Rust stable、Windows x64 的 MSVC 或 MinGW 工具链，或 Linux x64/arm64 的系统编译工具。运行 `cargo run -p hdsl`。

仓库发行脚本会在 `dist/` 生成 Windows 安装器和 ZIP、Linux `.deb` 和便携 tar.gz。Windows 可运行安装器，或解压 ZIP 后运行 `hdsl.exe`；Linux 可通过系统包管理器安装 `.deb`，或解压 tar.gz 后运行其中的 `hdsl`。便携包保留 `portable.flag`，数据会放在程序旁的 `data/`；安装版使用系统用户数据目录。

首次创建实例时，HDSL 从 Node.js 官方发行站下载 Node.js 24 LTS 并按官方 SHA256 校验，然后安装固定版本 pnpm 11.7.0 和所选的 Harness 版本。下载需要网络连接。HDSL 使用自己的数据目录，不使用系统已装的 Node、pnpm 或 dsh。

pnpm 可能提示某些依赖需要运行构建脚本。HDSL 会逐个显示精确包版本和 npm 声明的脚本命令，请审阅后选择“允许这次”或“拒绝”。拒绝会中止这次安装，并清理未完成的 Harness 目录。

Windows 数据目录为 `%LOCALAPPDATA%\hdsl-rs`，Linux 为 `${XDG_DATA_HOME:-~/.local/share}/hdsl-rs`。可执行文件旁存在 `portable.flag` 时，数据改存于同级 `data` 文件夹。

两类包均附带本手册的离线 HTML；点击启动器左侧“帮助文档”可在系统浏览器打开。Markdown 源文件也保留在仓库中。
