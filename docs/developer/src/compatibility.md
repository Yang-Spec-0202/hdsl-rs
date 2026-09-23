# 插件准入与兼容性

状态：兼容筛选、安装事务与构建脚本确认已实现；真实插件安装矩阵仍在验证。

目录 API 只提供候选，不能作为安装许可。候选必须有可验证的 GitHub 仓库、安装包与 `dsh.bundle.patch`。插件版本满足以下任一路径才显示安装按钮：

1. 包声明 `@deepseek-ai/*` peer dependency 范围，且所声明的每个包都存在于当前实例安装中并满足 npm 语义版本规则；
2. 仓库内人工维护的兼容记录列出插件精确版本及可用的 Harness 精确版本，附核验日期和证据。

未知、元数据不全或冲突的版本不提供安装。目录首先人工审查 DSH Plugin Marketplace、DSH Market、DSH Better Sidebar 和 OpenViking 插件。在线列表按“当前实例可安装版本”过滤；默认选最高兼容版本。

安装必须停止实例、备份目标 profile、调用该实例的 `dsh plugin`、执行配置转储检查。失败恢复备份。构建脚本逐次展示并取得用户确认，默认不放行。

验收：预发布版本范围、缺失 peer 包、目录伪装、安装失败和备份恢复各有自动化测试；未知版本没有强制安装入口。

## 安装事务与脚本确认（已实现）

安装前将整个旧 `web` profile 原子移动到同一文件系统的临时备份目录，再从备份复制非 `node_modules` 内容建立候选 profile。这样失败时可删除候选并将原 profile 原样移回。旧 profile 不存在时失败只删除新建目录。无论哪个安装步骤失败，恢复失败也必须同时报告两项错误。

首次调用官方 `dsh plugin` 时不批准构建脚本。pnpm 若报告 `ERR_PNPM_IGNORED_BUILDS`，解析其中每个精确包名和版本，从 npm 元数据读取该包 `preinstall`、`install`、`postinstall`、`prepare` 脚本并逐个向用户展示。用户拒绝、脚本缺失或元数据无法核验时立即回滚；用户确认后只将这个精确包版本加入候选 profile 的 `allowBuilds`，再调用官方命令。安装结束前用该实例的 `--dump-config` 检查配置。确认记录仅留在该实例 profile 中，不形成全局信任名单。

验收：现有 profile 的失败恢复要保留原目录内容；空 profile 失败不得留下残余；拒绝脚本时不执行脚本；多个脚本各有独立确认；日志与错误文字不得包含凭据。

## 已安装列表与卸载（已实现）

从当前实例的 `profiles/web/package.json` 读取插件包名，在插件页列出。卸载前停止实例，并沿用安装事务：保留整个旧 profile，候选目录执行官方 `dsh plugin --profile web remove <包名>`，再做配置转储检查。失败时原样恢复。界面必须为卸载提供明确确认，且切换实例后列表立即更新。

## 安装包内容核验（已实现）

调用 `dsh plugin` 前只接受 npm registry 的 HTTPS tarball URL 和 `sha512-...` integrity，下载上限 25 MiB。下载字节的 SHA512 必须与 registry 声明一致；压缩包内的 `package/package.json` 要与所选包名、精确版本和 `dsh.bundle.patch` 一致，声明的 patch 必须是包内普通文件。路径不得越过包根目录。任一检查失败则不创建候选 profile，也不执行包脚本。

## 首批目录审查记录

2026-09-23 按 npm 完整元数据与 GitHub 仓库地址核对四个候选。它们的发布包都有 `dsh.bundle.patch` 与 SHA512 integrity，但仍须逐版本满足当前实例实际安装的 `@deepseek-ai/*` peer 范围；目录收录本身不等于兼容。

| 仓库 / 包 | 当日 npm latest | 在 Harness `0.1.5-rc.3` 上的结果 |
| --- | --- | --- |
| `w2112515/dsh-plugin-marketplace` / `@w2112515/dsh-plugin-marketplace` | `0.2.4` | latest 的 `dsh-app-boot` 等 peer 要求 `^0.1.0-rc.6`；本次筛选无可安装版本。 |
| `dsh-market/dsh-market` / `dshmarket` | `1.58.0` | 筛选出兼容旧版 `1.11.3`，未将 latest 误荐。 |
| `omdsh-dev/DSH-better-sidebar` / `dsh-better-sidebar` | `0.19.1` | 无满足实例实际包版本集合的版本；保持不可安装。 |
| `volcengine/OpenViking` / `@openviking/dsh-memory-plugin` | `0.5.2` | `0.5.2` peer 匹配；在线测试完成包核验、安装、配置检查和卸载。 |

其中 npm 元数据是筛选证据，安装包内容另由 SHA512 和 `dsh.bundle.patch` 检查确认。当前人工精确版本矩阵为空；当 peer 证据不存在时，必须先补充带日期与证据的矩阵记录，不能靠“热门”推断兼容。

## tarball 全条目路径限制（已实现）

上面的包内容核验必须遍历压缩包的每一个条目，而不只看 manifest 与 patch。只接受 `package/` 根下的普通文件和目录；绝对路径、`..`、平台间含义不同的反斜杠路径以及符号链接、硬链接均拒绝。验收：即使合法的 manifest 和 patch 同时存在，额外的逃逸条目也使核验失败。
