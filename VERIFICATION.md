# 验证记录

## 已执行

环境：Android 手机上的 Ubuntu 24.04 / aarch64 / proot，Rust 1.90.0，Slint 1.14.1。

- `cargo test --locked --features ui`：34 项核心测试 + 1 项界面综合测试通过。
- `cargo clippy --locked --all-targets --features ui -- -D warnings`：通过。
- `cargo fmt --all -- --check`：通过。
- `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_verify_apk.py -v`：3 组测试通过，含 ELF、ZIP 和 manifest 元数据的正反例。它们是合成夹具，不是真实 APK。
- `bash -n scripts/build-android.sh`：通过；两个 Python 文件 AST 语法检查通过。

核心测试覆盖原示例 16.59、缓存率边界、全角/空白/百分号、非法数字/负数/非有限值/溢出、相对浮点容差、科学计数法、错误隔离和清空。

界面使用 Slint MinimalSoftwareWindow 无屏软件渲染，检查 390×844、320×640 和 390×360 的窗口，验证深浅主题、错误修复、关于开关、一键清空、实际指针点击、键盘输入事件及滚轮滚动。该检查不验证 Android 输入法、触屏驱动或系统边距。

## 已知限制与未执行项目

- 本机 aarch64 无法原生执行官方 Linux x86_64 Android SDK/NDK；未在本机生成 APK。
- GitHub Actions 工作流已提供并进行静态检查，但尚未在用户仓库实跑；不能据此声明 Android 编译或签名成功。
- 无屏软件渲染截图中部分中文字形存在重叠/块状瑕疵；指定 Noto 字体未解决，已撤回。不推断 Android Skia 渲染也有同样问题，需实际 APK 验证。
- 关于页 Slint 组件保留，页面留白不影响操作。
- 没有 Android 安装、启动、输入法、前后台、横屏、安全区或 16 KB 页面设备的端到端测试结果，见 ANDROID-CHECKLIST.md。
- 默认工作流每次创建新的临时测试签名；不是生产签名，跨次安装需先卸载旧版。
- 应用目标是 Android 8+ ARM64 直接安装，target SDK 34；不声称达到当前 Play 上架要求。

## 云端成功的判断标准

用户仓库 Actions 中所有步骤通过，Artifacts 中出现 APK、SHA256SUMS、BUILD-INFO 和 legal 目录；APK 的签名、manifest、ELF 与 ZIP 对齐检查实际通过，然后继续执行真机验收清单。源码测试通过不能替代这些检查。
