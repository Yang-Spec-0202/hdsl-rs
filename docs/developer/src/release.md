# 构建、测试与发行

状态：计划中。

CI 在 Windows x64、Linux x64 和 Linux arm64 构建与测试。核心测试使用假 registry 与假 `dsh`，覆盖下载校验、事务性安装、版本隔离、插件兼容及失败回滚；发行前以真实 Harness 和首批插件做人工端到端检查。

`mdbook build docs/developer` 与 `mdbook build docs/user` 必须通过，书籍缺页不得自动创建。本地链接和 Rust 代码示例需在 CI 检查。

Windows 发行 NSIS 安装器和 ZIP 便携包；Linux 发行 DEB 和 tar.gz。用户手册构建出的 HTML 随两类发行包附带，帮助按钮打开本地首页。发行时核对许可、致谢、SHA256 和 Git 标签。

