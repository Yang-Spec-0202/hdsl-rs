# 上游版本公告

## 2026-09-23：`0.1.5-rc.2` 的 Windows Web 启动异常

[DeepSeek Harness 官方仓库 Discussion #7593](https://github.com/deepseek-ai/deepseek-harness/discussions/7593) 有社区用户报告：`0.1.5-rc.2` 在 Windows 上启动默认 Web profile 时可能退出。HDSL 的本地测试复现了 `user patch-layer watching requires the Cordis HMR service`；另一种依赖解析结果会出现 `hmr.registerConfig is not a function`。

这是上游功能故障报告，**不是已确认的安全漏洞公告**。HDSL 不改写 Harness 的依赖或启动配置。若遇到该错误，请查看“运行日志”并在原讨论跟进；也可以新建实例试用其他精确版本，原实例及其 home 会保留。其他版本通过一次启动测试不代表所有环境或插件组合都已验证。

[Discussion #7448](https://github.com/deepseek-ai/deepseek-harness/discussions/7448) 另有社区用户报告 `0.1.5-rc.2` 在 npm 全局安装时可能因后续 RC 的依赖解析而失败；该安装方式与 HDSL 的实例安装不同，不能据此断定所有安装都会失败。2026-09-23 查阅的[官方安全公告页](https://github.com/deepseek-ai/deepseek-harness/security/advisories)没有已发布公告。

## 0.1.7 预览版

安装页会读取 npm 官方 `@deepseek-ai/dsh` 版本目录，并把 `alpha`、`beta`、`rc` 标为预览版。2026-09-23 核验时，[官方 GitHub Releases](https://github.com/deepseek-ai/deepseek-harness/releases) 包含 `0.1.7-alpha.1` 和 `0.1.7-alpha.2`；官方 npm 包目录还提供 `0.1.7-rc.1`，当时未在 GitHub Releases 列表看到对应条目。

HDSL 已在 Windows 测试实例中完成 `0.1.7-alpha.2` 和 `0.1.7-rc.1` 的安装、配置转储与 Web 启动检查。预览版可能改变插件接口；插件市场仍只提供有当前实例兼容证据的版本。
