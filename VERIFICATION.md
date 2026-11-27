# 验证记录

## v0.2.2 双模型版

模型数量从4改为2：表单与自动保存仅有两组，界面仅渲染模型1和模型2，关于文案同步调整。五项参数、原公式、排名、改名、保名清除、中文字体、拖动和外观设置保持不变；发布准备将项目和关于页版本号同步为0.2.2，不升级第三方依赖，不安装或卸载应用。

已运行 `cargo test --locked --features ui --test core` 62项，以及 `cargo test --locked --features ui --test storage --test ui --test ui_storage` 18项，总计80项全部通过。覆盖两模型轮流获胜/并列、空白与显式0、错误隔离、相对容差/微价差、第三/四组越界操作不改变状态或保存文件；界面仍检查真实指针/键入后的清屏、主题/字号/窄屏/拖动与重开恢复。严格Clippy（`--locked --all-targets --features ui -- -D warnings`）、rustfmt、diff空白和329个字面量的中文字体/OFL检查通过。

新增旧版四模型配置回归：加载保留前两组名称、原始参数、主题和字号，加载本身不改原文件；下一次保存只写两组。需要保留原第3、4组参数时应先自行记录。此前0.2.1双模型测试APK已由Actions46046658927构建并通过静态校验；本次0.2.2需重新云端构建，结果随Release的独立报告交付。Android真机仍未验收。

## v0.2.1 清空显示与输入框修复

用户反馈“清除此项 / 全部清空”无可见效果，以及输入框像两个重叠在一起。新增真实键入后检查编辑器文字的断言，在旧实现上复现：模型、排名已清空，但屏幕输入框仍显示旧值（`per-model clear must empty the visible editor after actual typing` 失败）。v0.2.0 的相关测试只检查了系数、排名或保存模型，未检查实际编辑器内容；旧版测试通过不能证明这个场景正确。

修复 PriceRow 的数值所有权：由模型单向供值，输入事件只回调 Rust，模型值变化时显式同步编辑器，避免双向绑定写入切断父级绑定。删除包裹标准 Fluent LineEdit 的额外描边，只保留标准控件的一层边框和焦点提示，错误仍由字段下方文字显示。不改计算、排名、字体、保存格式或签名体系，不升级第三方依赖。

本地 `cargo test --locked --features ui` 78 项通过（61 核心、15 存储、1 综合界面、1 保存联通）。新增回归包括实际键入→单清可见文字为空→再输入→聚焦时外部值同步→负数错误→全清→再次输入全清，以及保存文件清空、保留名称/设置、重开仍为空。已有字号/主题/窄屏/拖动回归保留；软件渲染中单清后旧文字消失，浅色和深色均仅有标准输入框外观。含 UI 的严格 Clippy（所有目标，警告视为错误）、rustfmt、diff 空白、329 个可见字面量的字体覆盖与 OFL 哈希、3 组 APK 校验器单元测试均通过。云端 APK 构建/下载结果随 Release 的独立报告交付，不将软件渲染等同于 Android 真机验收。

安装与真机限制不变：临时测试签名跨构建通常不能覆盖安装；卸载/清除数据会删除自动保存内容，无导入导出。未自动安装或卸载，Android 真机触摸/输入法与系统杀进程恢复仍需用户验收，见 ANDROID-CHECKLIST.md。旧 v0.1.1 / v0.1.2 / v0.2.0 保留不改。

## v0.2.0 四模型与本机保存（源码已验证）

参考上游固定提交 dbdf2917c0f29d80dd6e51a5e013835e3af75e7e，适配四模型、名称、独立清除、排名、即时本机保存、主题和 12～20 字号。本次保留内嵌 Noto 中文字体，不增加字体切换、快捷方式或导入导出。

本地已执行并通过：`cargo test --locked --features ui` 共 78 项（61 核心、15 存储、1 综合界面、1 界面与保存联通）；含 UI 的 Clippy `--all-targets -- -D warnings`、rustfmt 与 diff 空白检查通过；字体及 OFL 哈希、329 个 UI/Rust 字面量字符覆盖通过；3 组 APK 校验器单元测试和打包脚本语法检查通过。综合界面覆盖真实鼠标按下/移动/松开与键入事件、单清/全清、30 字名称及外部同步、320 宽/390×360 视口、12/20 字号、长中文名、主题、三页滚动到底与回顶；软件渲染截图另作目视复核。云端 Android 编译及取回 APK 复验结果随正式 Release 附件发布，不将源码测试等同于真机或 APK 通过。

核心覆盖原公式 6.84、缓存权重、非法/全角/溢出、相对 1e-12、微价差、四模型空白排除与错误隔离、稳定并列及真正最小值；存储覆盖原始输入往返、保名清除、损坏备份、未知版本先保护、大小限制、读写失败、原子替换和私有权限。

Android 入口只使用系统提供的应用内部私有目录，每次编辑立即同步保存，不依赖正常退出。不申请存储或联网权限。同步落盘在慢设备上的性能、真实输入法/触摸、系统栏、安全区、进程被杀后恢复、浏览器调用及 16KB 页面设备仍待真机验收；软件渲染与单元测试不等于这些项目通过。每次构建仍为新的临时测试签名，跨次通常不能覆盖安装；卸载/清除数据会删除已保存内容，本版没有迁移功能。旧 v0.1.1/v0.1.2 保持不变。

## v0.1.2 界面修复（已发布）

用户反馈 v0.1.1 中文缺字及不能触屏向下滑动。已在 Slint 1.14.1 Fluent ScrollView 源码确认其拖动默认关闭；新增按下/移动/松开回归后，旧配置在 `finger drag must move content` 断言失败，开启 `mouse-drag-pan-enabled` 后滑到底部、反向回顶通过。两个滚动区均启用；输入框起手拖动超过100px，以及390×360小视口到底/回顶的回归均通过。

中文方案为随包嵌入官方 Noto Sans CJK SC Regular v2.004（含 OFL 许可），不再只指定系统字体名字；字体及许可SHA256、实际字体族名、205个可显示字面量字符覆盖检查通过。36核心+1扩展UI测试、Clippy与格式检查通过；390×844首页和拖到底部截图中的中文已逐图检查，未见旧版重叠。本次版本号0.1.2，计算核心和第三方依赖版本不变。已正式发布：https://github.com/A-2788553793/relay-price-calc-m/releases/tag/v0.1.2 ，提交7e58014dbc6b83d460de675df2a4a7a8a1d56590，Actions45981435819成功。APK18645496字节，SHA256为2cc7ba375ca915da1c10f205fdf50a56a9f1b62db27e87919e8086bd1fe8e5d3；取回后完整字体/OFL、0.1.2版本、v2/v3签名、ARM64/API26、零权限与16KB静态对齐检查通过，五附件远端摘要一致。自动化模拟不能替代真机触摸与字体验收。

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
