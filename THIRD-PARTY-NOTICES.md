# 第三方软件与开源许可声明

本项目在开发与打包过程中使用了以下上游源码、界面框架、内嵌字体及平台依赖。CI 打包会自动将本文件与 `licenses/` 目录随产物一并分发。

---

## 1. 上游计价逻辑
- **来源**：[A-2788553793/relay-price-calc](https://github.com/A-2788553793/relay-price-calc)
- **许可协议**：MIT License（保留原作者 `Copyright (c) 2026 A-2788553793`，全文见 [LICENSE](./LICENSE)）。

## 2. Slint 1.14.1
- **版权声明**：Copyright © SixtyFPS GmbH `<info@slint.dev>`
- **许可协议**：Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0（本地副本见 [licenses/Slint-Royalty-free-2.0.md](./licenses/Slint-Royalty-free-2.0.md)，[上游原文](https://github.com/slint-ui/slint/blob/v1.14.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md)）。
- **署名提示**：应用「关于」页包含 Slint 官方 `AboutSlint` 署名组件，二次构建或分发时请保留该署名。

## 3. Noto Sans CJK SC 字体
- **随包文件**：`assets/fonts/NotoSansCJKsc-Regular.otf`（v2.004，未经修改）
- **许可协议**：SIL Open Font License 1.1（完整协议见 `assets/fonts/LICENSE` 与 `licenses/NotoSansCJK-OFL.txt`；上游来源与 SHA256 校验值见 [assets/fonts/README.md](./assets/fonts/README.md)）。
- **范围说明**：该字体独立遵循 OFL 1.1，不受本项目源码的 MIT License 约束。

## 4. 其他 Rust 与系统依赖
- 所有 Rust 传递依赖的精确版本均锁定于 [Cargo.lock](./Cargo.lock)。
- Android 目标使用 `android-activity`、`jni`、`Skia / skia-safe` 等组件；Linux 开发预览额外依赖 `winit` 与软件渲染器；中文主字形由随包字体渲染，其他缺失字符回退至系统字体。
- 上述依赖均各自保留原有版权与开源协议。如计划面向公众做大规模二进制发行，建议基于实际构建目标的完整依赖图补充详尽的第三方清单（包含 Skia 及其内嵌子依赖）。
