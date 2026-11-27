# 验证与交付记录

本文汇总 `relay-price-calc-m` 自初版移植至 **`v0.2.2`** 的自动化回归、静态检查与发布基线。

> **测试边界声明**：本地单元测试、Slint 软件渲染（`MinimalSoftwareWindow`）与 APK 静态分析**均不等于真机通过**。触屏手感、软键盘、安全区、进程强杀恢复及 16 KB 页面设备兼容性，均需按 [ANDROID-CHECKLIST.md](./ANDROID-CHECKLIST.md) 完成实机验收。

---

## 当前主线：v0.2.2（双模型版）

### 版本改动要点
- 对比组数由 4 组调整为 2 组（`MODEL_COUNT = 2`），主界面仅渲染「模型 1」「模型 2」，「关于」页版本号更新为 `0.2.2`。
- **旧配置向下兼容**：读取既有 4 模型 `config.json` 时，保留前两组的名称、原始输入及外观设置；加载过程不改写磁盘原文件，待下次编辑时再按双模型格式写回。

### 本地自动化回归（80 项 Rust 测试全过）
| 检查项 | 结果 | 覆盖重点 |
| :--- | :--- | :--- |
| `cargo test --locked --features ui --test core` | **62 通过** | `6.84` 基准公式、缓存率边界、全角 / 空白 / 溢出、`1e-12` 判等与微价差、两模型互胜 / 并列、第 3 / 4 组越界操作隔离 |
| `cargo test --locked --features ui --test storage --test ui --test ui_storage` | **18 通过** | 原子写入（`0o600`）、坏配置自动备份、未知 schema 防覆盖、真实键入后清屏同步、主题与字号缩放、窄屏拖动 |
| 格式 / Lint / 资源校验 | **全过** | `cargo fmt --all -- --check`、严格 Clippy（`--locked --all-targets --features ui -- -D warnings`）、diff 空白检查、`check-font.py`（329 个源字面量 cmap 覆盖与字体 / OFL SHA256） |

### 云端 CI 与发布复验
- **正式 Release（`fc7b8ca`）**：GitHub Actions `46050057180` 全绿，5 项附件匿名下载复验通过；APK 大小 `18,751,992` 字节，v2/v3 签名、`arm64-v8a` / API 26–34、零敏感权限及 16 KB ELF / ZIP 对齐静态检查全部通过。
- **主线合入**：`release/v0.2.2` 已快进合入 `main`，并触发 Actions `46052578216` 完成全套校验与测试包构建。

---

## 历史版本演进

| 版本 | 重点变更 | 核心验证记录 |
| :--- | :--- | :--- |
| **v0.2.1** | 修复真实键入后点击清除但屏幕残留旧字的问题（`PriceRow` 单向供值 + `changed value` 显式同步）；移除 `LineEdit` 外层多余边框。 | 78 项 Rust 测试（61 核心 + 15 存储 + 2 UI）、严格 Clippy、字体校验与 3 组合成 APK 校验器单测通过；Release 保留。 |
| **v0.2.0** | 参照上游 `dbdf2917` 实现 4 模型对比、自定义名称（30 字符）、保名清除、排名与节省比例、应用私有目录原子自动保存、浅 / 深主题与 `12–20` 字号。 | 78 项 Rust 测试通过，涵盖 `320` 窄屏与 `390×360` 视口软件渲染、三页滚到底 / 回顶及截图核对；Release 保留。 |
| **v0.1.2** | 解决 `v0.1.1` 中文缺字与页面不可拖动：随包嵌入 `NotoSansCJKsc-Regular.otf`（v2.004，`16,437,364` 字节），滚动视图开启 `mouse-drag-pan-enabled`。 | 36 核心 + 1 扩展 UI 测试（含输入框起手拖动 >100px 与 `390×360` 到底回顶）、字体哈希及 205 字面量检查通过；Release（`7e58014`，Actions `45981435819`）保留。 |
| **v0.1.1** | 对齐上游缓存加权公式 `(输入 + 输出) × (1 − r) + (缓存读取 + 缓存写入) × r`（示例 `6.84`），补齐逐项权重与胜负反转用例。 | 36 核心 + 1 UI 测试通过；Release（`9bc71bb`，Actions `45974441244`，APK `5,063,160` 字节）保留。 |
| **v0.1.0** | 完成 Rust + Slint Android 初始移植与 CI 适配；针对 `cargo-apk 0.10.0` 不支持 `--locked`，改用 `cargo fetch --locked` + `Cargo.lock` 哈希比对与 `check-build-cli.py` 护栏。 | 34 核心 + 1 UI 测试通过；第 5 次云端构建（`65b6004`，Actions `45951833346`）成功出包。 |

---

## 已知边界与出包准入

- **构建平台限制**：手机上的 `aarch64` Ubuntu / proot 无法运行官方 Linux x86_64 Android SDK / NDK，Android 产包统一依赖 GitHub Actions 或 x86_64 Linux 主机。
- **临时签名机制**：默认 CI 每次构建签发一次性测试证书，不同构建的 APK 无法覆盖安装，卸载旧包会清除本地数据。
- **候选包准入标准**：只有当 GitHub Actions 全绿、完整产出 `APK` / `SHA256SUMS` / `BUILD-INFO.txt` / `legal/` 并通过 `verify-apk.py` 静态校验时，才视为可进入真机验收的候选版本。
