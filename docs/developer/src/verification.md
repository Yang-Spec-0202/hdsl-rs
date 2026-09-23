# 验证记录

以下是特定日期和环境下的测试结果，不能代替当前版本的发行验证。上游版本和插件元数据可能随时变化。

## 上游版本（2026-09-23）

当时的 [GitHub Releases](https://github.com/deepseek-ai/deepseek-harness/releases) 包含 `0.1.7-alpha.1` 和 `0.1.7-alpha.2`；npm 目录还列出 `0.1.7-rc.1`，但当时未在 GitHub Releases 中看到对应条目。Windows 测试实例完成了 `0.1.7-alpha.2` 和 `0.1.7-rc.1` 的安装、配置转储与 Web 启动检查。`0.1.5-rc.2` 的 Web 启动出现 HMR 异常；复现过程依赖在线分发站和 npm，因此相关测试默认标记为 ignored。上游故障及处理方式另见用户手册的版本公告。

## 打包（2026-09-23）

2026-09-23 的本地验证：Windows GNU 目标生成 ZIP 和 NSIS 安装器，ZIP 内检查了程序、`help/index.html`、许可、原创图标与便携标记；Ubuntu 26.04 WSL x64 生成 tar.gz 和 `.deb`，核对了程序、离线帮助和图标路径，`ldd` 未报告缺失库。这不代替 CI 中的 Windows MSVC、Ubuntu 24.04 x64/arm64 验证。

## 插件候选（2026-09-23）

2026-09-23 按 npm 完整元数据与 GitHub 仓库地址核对四个候选。它们的发布包都有 `dsh.bundle.patch` 与 SHA512 integrity，但仍须逐版本满足当前实例实际安装的 `@deepseek-ai/*` peer 范围；目录收录本身不等于兼容。

| 仓库 / 包 | 当日 npm latest | 在 Harness `0.1.5-rc.3` 上的结果 |
| --- | --- | --- |
| `w2112515/dsh-plugin-marketplace` / `@w2112515/dsh-plugin-marketplace` | `0.2.4` | latest 的 `dsh-app-boot` 等 peer 要求 `^0.1.0-rc.6`；本次筛选无可安装版本。 |
| `dsh-market/dsh-market` / `dshmarket` | `1.58.0` | 筛选出兼容旧版 `1.11.3`，未将 latest 误荐。 |
| `omdsh-dev/DSH-better-sidebar` / `dsh-better-sidebar` | `0.19.1` | 无满足实例实际包版本集合的版本；保持不可安装。 |
| `volcengine/OpenViking` / `@openviking/dsh-memory-plugin` | `0.5.2` | `0.5.2` peer 匹配；在线测试完成包核验、安装、配置检查和卸载。 |

其中 npm 元数据是筛选证据，安装包内容另由 SHA512 和 `dsh.bundle.patch` 检查确认。当前人工精确版本矩阵为空；当 peer 证据不存在时，必须先补充带日期与证据的矩阵记录，不能靠“热门”推断兼容。
