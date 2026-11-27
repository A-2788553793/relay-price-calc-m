# relay-price-calc-m

基于 [A-2788553793/relay-price-calc](https://github.com/A-2788553793/relay-price-calc) 制作的 Rust + Slint Android 手机版。

## 功能

- 两个模型分别填写输入、输出、缓存写入、缓存读取单价及平均缓存率，可自定义名称（最多 30 个字符）。
- 即时显示价格系数、排名、并列最便宜模型，以及最便宜模型相对各较贵模型的节省比例。
- 整组全空不参与比较；参与模型中部分字段空白按 0，显式填写 0 的模型会参与比较。支持全角数字、句号、科学计数法和末尾百分号。
- 无效数字、负数、缓存率超过 100% 和运算溢出均有提示；只排除有误模型，其他有效模型仍可比较。至少两个有效模型才给出比较结论。
- 单独清除模型或清空全部参数；清除均保留模型名称和设置。
- 双模型纵向触屏拖动（包含输入框区域起手）、数字键盘提示；设置页提供浅色/深色及 12～20 字号，默认 14。
- 随包嵌入原有完整 Noto Sans CJK SC 中文字体；本次不新增字体切换或快捷方式。
- 每次编辑自动保存原始参数、名称、主题和字号到应用私有目录；同一安装内重开恢复，不需要存储权限。损坏配置备份、未知版本保护及写失败提示，不把保存失败显示为成功。
- 计算与保存完全离线，无账号、无联网权限；关于页保留 Slint 官方署名，并可调用系统浏览器打开 GitHub（打开网站需要浏览器联网）。

本次按上游 [v0.2.0 / dbdf2917](https://github.com/A-2788553793/relay-price-calc/tree/dbdf2917c0f29d80dd6e51a5e013835e3af75e7e) 适配 Android，不照搬 Windows 字体和桌面快捷方式。

## 计算口径

沿用原项目公式，**不是实际请求账单估算器**。参与比较的模型须使用相同货币和单位（建议每百万 token）。缓存率填 `80` 表示 80%，不是 0.8。

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

**不同运行的 APK 通常不能覆盖安装**：证书变化会导致“应用未安装/签名不一致”。若需要卸载旧版，请先自行记录所需参数；**卸载或清除应用数据会删除自动保存的所有内容，本版没有导入导出**。自动保存仅保证同一安装内重开恢复，不代表跨卸载迁移。正式分发前应自行配置长期安全保存并备份的签名证书。

工作流不自动发布 Release、不推送代码、不上传 keystore，仅上传 APK、SHA256 和许可等构建产物。

## 开发测试

固定 Rust 1.90.0、Slint 1.14.1，提交 Cargo.lock。

```bash
python3 scripts/check-font.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_apk.py -v
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets --features ui -- -D warnings
cargo test --locked --features ui
# 桌面仅作开发预览，不是首版交付平台
cargo run --locked --features ui
```

Ubuntu 依赖：build-essential pkg-config libfontconfig1-dev libfreetype6-dev libxkbcommon-dev libx11-dev libxcb1-dev libx11-xcb-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libegl1-mesa-dev libgles2-mesa-dev。中文字体已嵌入；无需另装系统中文字体。

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

## v0.2.2 双模型版

v0.2.2 将模型数量由四个改为两个，界面、比较和自动保存均使用前两组。读取旧版四模型配置时保留前两组名称、原始输入和外观设置；第 3、4 组不再参与，下一次自动保存只写入两组。需要保留后两组数据时请先记录；临时测试签名跨构建通常不能覆盖安装，卸载会清空全部保存内容。本次不升级第三方依赖。

## 验证状态

v0.2.1 修复实际键入后“清除此项 / 全部清空”只清数据、未清屏幕文字的问题，并移除输入框重复外框，保留标准控件的边框和焦点提示。补充键入→清除→可见文字 / 保存文件 / 重开恢复的联通回归；该已发布版本继续保留 v0.2.0 四模型、排名、自动保存和设置，以及原公式、中文字体与触屏拖动；v0.2.2 在此基础上调整为双模型，见上节。APK 和正式发布状态见 [Releases](https://github.com/A-2788553793/relay-price-calc-m/releases)。详细验证和未测项目见 VERIFICATION.md 与 ANDROID-CHECKLIST.md。

软件渲染、单元测试及 APK 静态校验不等于 Android 真机验收：输入法、真实触摸、系统栏、进程终止后的恢复和外部浏览器仍需设备检查。助手不会自动卸载或安装应用。
