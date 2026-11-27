# relay-price-calc-m

基于 [A-2788553793/relay-price-calc](https://github.com/A-2788553793/relay-price-calc) 制作的 Rust + Slint Android 手机版。

## 功能

- 两个模型的输入、输出、缓存写入、缓存读取单价及平均缓存率。
- 即时显示价格系数、便宜的一方和节省比例（以较贵模型为基准）。
- 空白按 0；计算核心支持全角数字、句号和末尾百分号。
- 无效数字、负数、缓存率超过 100% 和运算溢出均有提示。
- 纵向滚动布局、数字键盘提示、浅色/深色、一键清空。
- 顶栏“关于”页包含 Slint 官方署名组件。
- 完全离线，无账号、无网络请求、无联网权限；不保存输入，关闭后需重新填写。

## 计算口径

沿用原项目公式，**不是实际请求账单估算器**。两个模型须使用相同货币和单位（建议每百万 token）。缓存率填 `80` 表示 80%，不是 0.8。

`价格系数 = (输入 + 输出) × (1 − r) + (缓存读取 + 缓存写入) × r`

r = 平均缓存率 / 100。示例输入 3、输出 15、缓存写入 3.75、缓存读取 0.3、缓存率 80，系数为 **6.84**。缓存率 0% 时只计输入与输出，100% 时只计缓存读取与写入。

浮点比较沿用相对容差 1e-12；普通系数至多 4 位小数，非零且小于 0.0001 时用科学计数法。胜负用原始值，因此显示系数相同仍可能有微小价差。

## Android 支持

- Android 8.0（API 26）及以上、ARM64（arm64-v8a）。
- 不包含 32 位 ARM 或 x86 模拟器 APK。
- 包名 io.github.a2788553793.relaypricecalcm。
- target SDK 34 是直接安装的兼容基线，**不表示符合当前 Google Play 上架要求**。
- 工作流检查 16 KB ELF 与 APK 对齐；实际键盘、导航栏、系统中文字体和生命周期仍需真机验收。

## GitHub Actions 生成 APK

1. 自己创建 GitHub 仓库，建议名 relay-price-calc-m。
2. 解压源码，将**项目文件夹内全部内容**提交到仓库根目录，不要再套一层目录。确认 Cargo.toml 和 .github/workflows/android.yml 都在；文件管理器可能隐藏 .github，切勿漏传。网页上传难以选择隐藏目录时使用 Git 或 GitHub Codespaces。
3. 打开 **Actions → Android APK (test signed) → Run workflow**。main/master 分支推送也触发；首次可能要启用 Actions。
4. 等步骤全部变绿，在本次运行的 **Artifacts** 下载 relay-price-calc-m-arm64-test-apk，解压取得 relay-price-calc-m-arm64-test.apk。
5. 将 APK 传到手机，仅对可信来源允许安装未知应用后安装。

首次下载 Rust/NDK/Slint/Skia 可能十几到几十分钟，实际取决于 GitHub 网络与缓存，不保证时长。

### 签名说明（重要）

默认每次运行产生**临时测试证书**，无需你的 Token、密码或正式密钥。它生成可安装测试 APK，不是生产签名。

**不同运行的 APK 通常不能覆盖安装**：证书变化会导致“应用未安装/签名不一致”。先卸载旧版再装新版；本应用不持久保存输入。正式分发前应自行配置长期安全保存并备份的签名证书。

工作流不自动发布 Release、不推送代码、不上传 keystore，仅上传 APK、SHA256 和许可等构建产物。

## 开发测试

固定 Rust 1.90.0、Slint 1.14.1，提交 Cargo.lock。

```bash
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_apk.py -v
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets --features ui -- -D warnings
cargo test --locked --features ui --test ui
# 桌面仅作开发预览，不是首版交付平台
cargo run --locked --features ui
```

Ubuntu 依赖：build-essential pkg-config libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libx11-dev libxcb1-dev libx11-xcb-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libegl1-mesa-dev libgles2-mesa-dev。截图可用 fonts-noto-cjk。

设置 RELAY_UI_SNAPSHOTS=1 运行 UI 测试可将真实软件渲染输出到 target/ui-shots/*.ppm；它们不是 Android 真机截图。

### Linux x86_64 本地 Android 打包

需 JDK17、SDK platform34/build-tools35.0.0、NDK28.2.13676358、Rust Android ARM64 target、cargo-apk0.10.0。设置 ANDROID_HOME、ANDROID_NDK_ROOT、ANDROID_NDK 后：

```bash
rustup target add aarch64-linux-android
cargo install --locked cargo-apk --version 0.10.0
python3 scripts/check-build-cli.py
bash scripts/build-android.sh
```

输出到 dist/。安装与环境顺序以 .github/workflows/android.yml 为准。手机 aarch64 Ubuntu/proot 不能直接运行官方 Linux x86_64 NDK，所以首版选择云端构建。

## 许可

代码为 MIT，保留原作者 Copyright (c) 2026 A-2788553793，见 LICENSE。
Slint 使用 Royalty-free Desktop, Mobile, and Web Applications License 2.0；保留“关于”内 AboutSlint 组件。第三方依赖各自许可见 THIRD-PARTY-NOTICES.md 和 licenses/。

## 验证状态

**v0.1.1 已同步上游新公式。新版 APK 构建与发布状态请查看本次 Actions 及 [Releases](https://github.com/A-2788553793/relay-price-calc-m/releases)。以下第 5 次构建是旧公式版本，不包含本次更新。**

[历史：GitHub Actions 第 5 次构建已成功并生成 APK](https://github.com/A-2788553793/relay-price-calc-m/actions/runs/45951833346#artifacts)。登录 GitHub，在 Artifacts 下载 `relay-price-calc-m-arm64-test-apk` 并解压安装。详细证据见 VERIFICATION.md；**构建与签名检查通过不等于 Android 真机验收通过**。
