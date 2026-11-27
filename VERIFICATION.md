# 验证记录

## v0.1.2 界面修复（进行中）

用户反馈 v0.1.1 中文缺字及不能触屏向下滑动。已在 Slint 1.14.1 Fluent ScrollView 源码确认其拖动默认关闭；新增按下/移动/松开回归后，旧配置在 `finger drag must move content` 断言失败，开启 `mouse-drag-pan-enabled` 后滑到底部、反向回顶通过。两个滚动区均启用；输入框起手拖动超过100px，以及390×360小视口到底/回顶的回归均通过。

中文方案为随包嵌入官方 Noto Sans CJK SC Regular v2.004（含 OFL 许可），不再只指定系统字体名字；字体及许可SHA256、实际字体族名、205个可显示字面量字符覆盖检查通过。36核心+1扩展UI测试、Clippy与格式检查通过；390×844首页和拖到底部截图中的中文已逐图检查，未见旧版重叠。本次版本号0.1.2，计算核心和第三方依赖版本不变。尚未生成本版本APK；自动化模拟不能替代真机触摸与字体验收。

## v0.1.1 新公式发布（已完成）

正式 Release：https://github.com/A-2788553793/relay-price-calc-m/releases/tag/v0.1.1 。提交9bc71bbc86db2e42318029c02f38d108eeaae27a，Actions45974441244成功；36核心+1UI、lint、打包与校验通过。已取回 APK 复验版本0.1.1、v2/v3签名、零权限、ARM64/API26/34及4个PT_LOAD段16KB对齐；5063160字节，SHA2563177d9f27f7a3cf6af724fd11ec92049809445c7c9657562591443fe3c678c39。匿名下载HTTP200，但当时未真机验收；用户随后报告的缺字与拖动问题即本次修复目标。

## 新公式同步（历史源码验证）

已按上游 src/calc.rs 的新公式同步：`(输入 + 输出) × (1 − r) + (缓存读取 + 缓存写入) × r`。示例变为6.84，0%为18，100%为4.05。本轮 `cargo test --locked --features ui` 已通过36项核心测试和1项界面综合测试；新增逐项价格权重及胜负反转回归，并更新缓存率0/25/80/100%、全角输入、逐字段重算、相等比较和溢出夹具。其后已完成v0.1.1新公式APK发布，见上文；历史第5次成功运行不能作为新公式APK验证。

本轮其余检查：Clippy `--all-targets --features ui -- -D warnings`、rustfmt检查、3组APK校验器测试和打包脚本 `bash -n` 全部通过。APK校验器使用合成夹具，不表示新APK已生成。

同步来源：https://raw.githubusercontent.com/A-2788553793/relay-price-calc/master/src/calc.rs（本次直接读取核对）。保持与上游相同的“先求和、再乘权重”运算顺序；即使某组权重为0，若该组求和溢出，也会报错。公式同步未修改依赖和Android工作流；v0.1.1发布准备仅更新Cargo.toml与Cargo.lock中的本项目版本号，未升级第三方依赖。

## 历史版本已执行

环境：Android 手机上的 Ubuntu 24.04 / aarch64 / proot，Rust 1.90.0，Slint 1.14.1。

- `cargo test --locked --features ui`：34 项核心测试 + 1 项界面综合测试通过。
- `cargo clippy --locked --all-targets --features ui -- -D warnings`：通过。
- `cargo fmt --all -- --check`：通过。
- `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_apk.py -v`：3 组测试通过，含 ELF、ZIP 和 manifest 元数据的正反例。它们是合成夹具，不是真实 APK。
- `bash -n scripts/build-android.sh`：通过；两个 Python 文件 AST 语法检查通过。

核心测试覆盖原示例 16.59、缓存率边界、全角/空白/百分号、非法数字/负数/非有限值/溢出、相对浮点容差、科学计数法、错误隔离和清空。

界面使用 Slint MinimalSoftwareWindow 无屏软件渲染，检查 390×844、320×640 和 390×360 的窗口，验证深浅主题、错误修复、关于开关、一键清空、实际指针点击、键盘输入事件及滚轮滚动。该检查不验证 Android 输入法、触屏驱动或系统边距。

## 历史旧公式版本：云端 APK 构建成功

- 仓库：https://github.com/A-2788553793/relay-price-calc-m
- 成功运行：[Android APK 第 5 次运行](https://github.com/A-2788553793/relay-price-calc-m/actions/runs/45951833346)，提交 `65b600474cb17de20a7826e3e2730f8b281863ad`，运行时长 6 分 18 秒。
- 已逐步核对公开任务状态：核心测试与 lint、桌面 UI 测试与 lint、CLI 回归、Android APK 构建校验、产物上传均为 success。
- 构建脚本启用 `set -euo pipefail`；签名验证、16 KB ELF/ZIP 对齐、ARM64 ABI、包名/版本基线及零权限检查为必过步骤。本次是云端检查通过，并非本机重新检查下载后的二进制。
- 产物名：`relay-price-calc-m-arm64-test-apk`，大小约 4.81 MB，artifact ID `15177256826`。下载解压得到 `relay-price-calc-m-arm64-test.apk`，并包含 SHA256SUMS、BUILD-INFO 及 legal 目录。
- GitHub 公开页面记录的产物 ZIP 摘要为 `sha256:6d552139a9fbf4c476bf76e5f9e31689d84b5c00c9e6433ace79a320d8e524e2`；它不是 APK 文件自身的摘要。
- 下载入口：[本次运行的 Artifacts](https://github.com/A-2788553793/relay-price-calc-m/actions/runs/45951833346#artifacts)。需登录 GitHub；匿名下载被拒绝，本机未取回此二进制。
- 已比对该提交的项目源码：24 个文件与本地交付一致，只有 README.md 和本验证记录不同；全部构建关键文件一致。

## 已知限制与未执行项目

- 本机 aarch64 无法原生执行官方 Linux x86_64 Android SDK/NDK；未在本机生成 APK。
- 历史问题：第 3 次云端运行打包退出 2；第 5 次已修复并生成 APK。
- 在本机使用 cargo-apk 0.10.0 复现：`cargo apk build --locked …` 和 `cargo apk --version` 均因不支持的参数退出 2。已改为 `cargo fetch --locked` + 打包前后 Cargo.lock 哈希检查，以及 `cargo apk version`。新增 `python3 scripts/check-build-cli.py`，直接检查真实脚本中的全部 cargo-apk 命令，修正后通过。该 CLI 检查本身不执行 Android 编译；修正版已在第 5 次云端运行实跑成功。
- 历史无屏软件渲染截图中文字形重叠，单独指定系统 Noto 字体名未解决；v0.1.1 用户也反馈字体异常。v0.1.2 改为真实嵌入字体，修复后的证据见本文件开头，不将旧版问题记作通过。
- 关于页 Slint 组件保留，页面留白不影响操作。
- 没有 Android 安装、启动、输入法、前后台、横屏、安全区或 16 KB 页面设备的端到端测试结果，见 ANDROID-CHECKLIST.md。
- 默认工作流每次创建新的临时测试签名；不是生产签名，跨次安装需先卸载旧版。
- 应用目标是 Android 8+ ARM64 直接安装，target SDK 34；不声称达到当前 Play 上架要求。

## 云端成功的判断标准

用户仓库 Actions 中所有步骤通过，Artifacts 中出现 APK、SHA256SUMS、BUILD-INFO 和 legal 目录；APK 的签名、manifest、ELF 与 ZIP 对齐检查实际通过，然后继续执行真机验收清单。源码测试通过不能替代这些检查。
