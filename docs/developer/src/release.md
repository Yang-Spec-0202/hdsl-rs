# 构建、测试与发行

状态：打包脚本与 CI 配置已实现；Windows x64、Linux x64 本地构建已验证，Linux arm64 等待 CI 原生运行器验证。

流水线固定使用 `Cargo.lock`、mdBook 0.5.3，并在 Windows x64、Linux x64/arm64 上分别检查 Rust 格式、Clippy、测试、两本 mdBook 的 HTML、Rust 示例和内部链接。打包脚本只消费编译后的二进制、原创图标、许可文件和离线用户手册。Windows 脚本生成 ZIP 与 NSIS 安装器；Linux 脚本生成 tar.gz 与 `.deb`。CI 在发行包内核对这些文件存在。

CI 在 Windows x64、Linux x64 和 Linux arm64 构建与测试。核心测试使用假 registry 与假 `dsh`，覆盖下载校验、事务性安装、版本隔离、插件兼容及失败回滚；发行前以真实 Harness 和首批插件做人工端到端检查。

`mdbook build docs/developer` 与 `mdbook build docs/user` 必须通过，书籍缺页不得自动创建。本地链接和 Rust 代码示例需在 CI 检查。

Windows 发行 NSIS 安装器和 ZIP 便携包；Linux 发行 DEB 和 tar.gz。用户手册构建出的 HTML 随两类发行包附带，帮助按钮打开本地首页。发行时核对许可、致谢、SHA256 和 Git 标签。

本地发行命令：Windows 运行 `scripts/package-windows.ps1`（需要 Rust、mdBook 0.5.3、NSIS），Linux 运行 `bash scripts/package-linux.sh`（需要 Rust、mdBook 0.5.3、`dpkg-deb`）。脚本先构建 release 二进制与用户手册，再在 `dist/` 输出平台包和 `SHA256SUMS`。便携包含 `portable.flag`；安装包不含该标记，因此两者的数据目录不会混用。

2026-09-23 的本地验证：Windows GNU 目标生成 ZIP 和 NSIS 安装器，ZIP 内检查了程序、`help/index.html`、许可、原创图标与便携标记；Ubuntu 26.04 WSL x64 生成 tar.gz 和 `.deb`，核对了程序、离线帮助和图标路径，`ldd` 未报告缺失库。这不代替 CI 中的 Windows MSVC、Ubuntu 24.04 x64/arm64 验证。
