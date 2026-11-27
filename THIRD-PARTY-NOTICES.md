# 第三方软件说明

## 原项目

本应用计算规则改编自 https://github.com/A-2788553793/relay-price-calc 。原作者 Copyright (c) 2026 A-2788553793，MIT License 全文见根目录 LICENSE。

## Slint 1.14.1

Copyright © SixtyFPS GmbH <info@slint.dev>。Slint 提供多许可，本 Android 应用采用 Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0，全文见 licenses/Slint-Royalty-free-2.0.md。应用顶栏可打开“关于”，包含 Slint 官方 AboutSlint 组件，请勿删除署名。

许可原文：https://github.com/slint-ui/slint/blob/v1.14.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md

## 其他依赖

Rust 传递依赖版本固定于 Cargo.lock。Android 使用 android-activity、jni、Skia/skia-safe 等，Linux 开发预览还使用 winit、software renderer；系统字体由操作系统提供，不随本项目分发。各依赖保有其自身的版权和许可。

此说明不替代各依赖的许可。正式分发二进制前应按实际目标的依赖图整理完整 notices（包括 Skia 及其内嵌依赖），保留其要求的版权声明。默认工作流一并上传本文件和 licenses/，但不声称已完成生产分发的完整法律审计。
