# 验证记录

## 新公式同步（当前源码）

已按上游 src/calc.rs 的新公式同步：`(输入 + 输出) × (1 − r) + (缓存读取 + 缓存写入) × r`。示例变为6.84，0%为18，100%为4.05。本轮 `cargo test --locked --features ui` 已通过36项核心测试和1项界面综合测试；新增逐项价格权重及胜负反转回归，并更新缓存率0/25/80/100%、全角输入、逐字段重算、相等比较和溢出夹具。尚未构建含新公式的APK；历史第5次成功运行不能作为本次APK验证。

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
- 无屏软件渲染截图中部分中文字形存在重叠/块状瑕疵；指定 Noto 字体未解决，已撤回。不推断 Android Skia 渲染也有同样问题，需实际 APK 验证。
- 关于页 Slint 组件保留，页面留白不影响操作。
- 没有 Android 安装、启动、输入法、前后台、横屏、安全区或 16 KB 页面设备的端到端测试结果，见 ANDROID-CHECKLIST.md。
- 默认工作流每次创建新的临时测试签名；不是生产签名，跨次安装需先卸载旧版。
- 应用目标是 Android 8+ ARM64 直接安装，target SDK 34；不声称达到当前 Play 上架要求。

## 云端成功的判断标准

用户仓库 Actions 中所有步骤通过，Artifacts 中出现 APK、SHA256SUMS、BUILD-INFO 和 legal 目录；APK 的签名、manifest、ELF 与 ZIP 对齐检查实际通过，然后继续执行真机验收清单。源码测试通过不能替代这些检查。
