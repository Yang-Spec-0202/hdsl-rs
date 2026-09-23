# 安装与启动

当前开发预览版可从仓库构建。需要 Rust stable、Windows x64 的 MinGW 工具链或 Linux x64/arm64 的系统编译工具。运行 `cargo run -p hdsl`。

首次创建实例时，HDSL 从 Node.js 官方发行站下载 Node.js 24 LTS 并按官方 SHA256 校验，然后安装固定版本 pnpm 11.7.0 和所选的 Harness 版本。下载需要网络连接。HDSL 使用自己的数据目录，不使用系统已装的 Node、pnpm 或 dsh。

Windows 数据目录为 `%LOCALAPPDATA%\hdsl-rs`，Linux 为 `${XDG_DATA_HOME:-~/.local/share}/hdsl-rs`。可执行文件旁存在 `portable.flag` 时，数据改存于同级 `data` 文件夹。

发行包和离线帮助页仍在制作中。开发构建可先在仓库中阅读本手册的 Markdown 源文件。
