# relay-price-calc-m

> 基于 [A-2788553793/relay-price-calc](https://github.com/A-2788553793/relay-price-calc) 构建的 Rust + Slint Android 中转站模型价格比较工具。

当前主线版本为 **v0.2.2（双模型版）**，提供完全离线的模型单价系数对比、实时排名、本地自动记忆及浅色/深色外观定制。

---

## 核心特性

- **双模型即时对比**：支持填写两个模型的「输入、输出、缓存写入、缓存读取」单价与「平均缓存率」，即时计算价格系数、排名、并列判定与最便宜模型的节省比例。
- **自定义名称与容错解析**：模型名可自定义（上限 30 字符，留空自动回退为 `模型1` / `模型2`）；支持全角数字、句号、科学计数法及末尾 `%`。
- **空值与异常隔离**：整组全空不参与排名，已启用模型中的空白字段按 `0` 计；非法数字、负数、超 `100%` 缓存率或算术溢出仅标红隔离当前模型，不影响另一模型计算。
- **保名清除与自动保存**：支持单模型「清除此项」与顶部「全部清空」，清除后保留模型名称与外观设置；每次编辑原子写入应用私有目录 `config.json`，损坏文件自动备份，拒绝覆盖未知新版本 schema。
- **移动端适配与外观**：主界面、设置与关于页均支持触屏纵向拖动；提供浅色/深色主题切换与 `12–20`（默认 `14`）字号缩放；随包嵌入完整 `Noto Sans CJK SC` 中文字体。
- **零敏感权限**：完全离线运行，不申请网络与外部存储权限；仅「关于」页通过系统 `Intent` 调起默认浏览器访问 GitHub 主页。

---

## 计算口径

本工具严格沿用上游项目公式（仅用于横向比较价格系数，**并非真实单次请求账单估算器**）。比较双方应采用相同货币与计价单位（建议每百万 token）；缓存率填写 `80` 即代表 `80%`。

```text
价格系数 = (输入 + 输出) × (1 − r) + (缓存读取 + 缓存写入) × r
其中 r = 平均缓存率 / 100
```

- **示例**：输入 `3`、输出 `15`、缓存写入 `3.75`、缓存读取 `0.3`、缓存率 `80` → 系数为 **`6.84`**（0% 时为 `18`，100% 时为 `4.05`）。
- **精度与显示**：以 `1e-12` 相对容差判等；常规数值最多显示 4 位小数，小于 `0.0001` 的非零值使用科学计数法，胜负比较基于原始浮点值。

---

## 平台规格与版本提示

| 项目 | 说明 |
| :--- | :--- |
| **系统要求** | Android 8.0（API 26）及以上，仅 `arm64-v8a`（target SDK 34，面向直接侧载，不代表 Google Play 上架合规） |
| **应用包名** | `io.github.a2788553793.relaypricecalcm` |
| **v0.2.2 变更** | 由 `v0.2.0/v0.2.1` 的 4 模型调整为 2 模型；加载旧版 4 模型配置时保留前两组数据与设置，下次保存仅写入两组 |
| **签名与数据** | CI 默认每次生成**临时测试签名**，不同构建之间通常无法直接覆盖安装；卸载或清除数据会清空本地保存，本版无导入/导出 |

---

## 获取与构建

### 方式一：GitHub Releases / Actions（推荐）
1. 直接下载正式安装包：前往 [GitHub Releases](https://github.com/A-2788553793/relay-price-calc-m/releases) 获取 `v0.2.2` 构建产物。
2. 自行触发云端构建：将仓库完整内容（含 `.github/workflows/android.yml`）推送至个人仓库后，在 **Actions → Android APK (test signed)** 中运行工作流，并在 Artifacts 下载 `relay-price-calc-m-arm64-test-apk`。

### 方式二：本地测试与桌面预览（Linux）
工具链固定为 **Rust 1.90.0** 与 **Slint 1.14.1**（锁定于 `Cargo.lock`）：

```bash
# 资源与构建脚本检查
python3 scripts/check-font.py
PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_apk.py -v

# Rust 单元测试、格式、Lint 与桌面 UI 回归
cargo fmt --all -- --check
cargo test --locked --no-default-features
cargo clippy --locked --all-targets --features ui -- -D warnings
cargo test --locked --features ui

# 启动桌面软件渲染预览（仅用于开发调试）
cargo run --locked --features ui
```

> 若运行 `RELAY_UI_SNAPSHOTS=1 cargo test --locked --features ui`，可将软件渲染截图输出至 `target/ui-shots/*.ppm`。桌面测试依赖 `libfontconfig1-dev`、`libfreetype6-dev`、`libxkbcommon-dev`、`libegl1-mesa-dev` 等系统开发包。

### 方式三：Linux x86_64 本地打包 Android APK
> 注：`aarch64` 手机 Ubuntu/proot 环境无法运行官方 x86_64 Android NDK，需在 x86_64 Linux 主机执行。

需要准备 JDK 17、Android SDK（`platforms;android-34`、`build-tools;35.0.0`）、NDK `28.2.13676358`、`aarch64-linux-android` target 及 `cargo-apk 0.10.0`：

```bash
rustup target add aarch64-linux-android
cargo install --locked cargo-apk --version 0.10.0
python3 scripts/check-build-cli.py
bash scripts/build-android.sh
```

打包结果与校验信息将输出至 `dist/` 目录。

---

## 文档导航与验证边界

- [ANDROID-CHECKLIST.md](./ANDROID-CHECKLIST.md)：Android 真机待验收清单（触摸、输入法、安全区、系统栏与生命周期）。
- [VERIFICATION.md](./VERIFICATION.md)：各版本单元测试、软件渲染及 APK 静态校验记录。
- [THIRD-PARTY-NOTICES.md](./THIRD-PARTY-NOTICES.md)：第三方开源许可与字体声明。

> **验证边界声明**：本地单元测试、软件渲染截图与 16 KB ELF/ZIP 静态校验均**不能替代 Android 真机实测**。

---

## 开源许可

- **应用源码**：MIT License（保留原作者 Copyright (c) 2026 A-2788553793，详见 [LICENSE](./LICENSE)）。
- **GUI 框架**：Slint 1.14.1 采用 Royalty-free Desktop, Mobile, and Web Applications License 2.0，保留「关于」页 `AboutSlint` 官方署名。
- **内嵌字体**：`NotoSansCJKsc-Regular.otf` 遵循 SIL Open Font License 1.1（不受本项目 MIT 许可覆盖）。
