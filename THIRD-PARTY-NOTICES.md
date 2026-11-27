# 第三方软件与许可说明

## 1. 上游项目
本应用计价规则改编自 [A-2788553793/relay-price-calc](https://github.com/A-2788553793/relay-price-calc)，遵循 MIT License（保留原作者 `Copyright (c) 2026 A-2788553793`，全文见根目录 [LICENSE](./LICENSE)）。

## 2. Slint 1.14.1
- **版权所有**：Copyright © SixtyFPS GmbH `<info@slint.dev>`
- **适用许可**：Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0（全文见 [licenses/Slint-Royalty-free-2.0.md](./licenses/Slint-Royalty-free-2.0.md)，[上游原文](https://github.com/slint-ui/slint/blob/v1.14.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)）。
- **署名要求**：应用「关于」页保留了 Slint 官方 `AboutSlint` 组件，二次分发时请勿移除。

## 3. Noto Sans CJK SC
- **嵌入字体**：`assets/fonts/NotoSansCJKsc-Regular.otf`（v2.004，未修改）
- **适用许可**：SIL Open Font License 1.1（完整声明见 `assets/fonts/LICENSE` 与 `licenses/NotoSansCJK-OFL.txt`；来源与 SHA256 见 [assets/fonts/README.md](./assets/fonts/README.md)）。
- **说明**：该字体受 OFL 1.1 约束，不受本项目 MIT 许可覆盖。

## 4. 其他 Rust 与平台依赖
- Rust 传递依赖版本锁定于 [Cargo.lock](./Cargo.lock)。
- Android 构建依赖 `android-activity`、`jni`、`Skia/skia-safe` 等组件；Linux 开发预览还使用 `winit` 与软件渲染器；中文主字形由内嵌字体提供，其余缺失字符回退由系统提供。
- 上述各组件均保留其原有版权与开源许可。正式向公众大规模分发二进制产物前，应按实际目标依赖图梳理完整第三方声明（含 Skia 及其内嵌子依赖）；CI 工作流会随产物一并打包本文件与 `licenses/` 目录。
