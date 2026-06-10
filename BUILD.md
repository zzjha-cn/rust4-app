# Blink Reminder 打包与分发指南

本文档说明了如何将 Blink Reminder 编译并打包为可分发的独立应用程序（如 macOS 的 `.app` 和 `.dmg`，Windows 的 `.exe` 和 `.msi`）。

## 1. 环境准备

在打包之前，请确保你的系统已安装以下依赖：

- **Node.js** (推荐 v18+)
- **Rust** (推荐 1.78+)
- **Cargo** (随 Rust 一起安装)

## 2. 性能与体积优化配置

为了让最终的应用程序占用极低的内存和磁盘空间，我们在 `src-tauri/Cargo.toml` 中配置了 Release 模式的极限优化参数：

```toml
[profile.release]
panic = "abort"        # 禁用 panic 展开，减小体积
codegen-units = 1      # 最大化优化（编译变慢，但运行更快、内存更小）
lto = true             # 开启链接时优化
opt-level = "s"        # 针对体积进行优化
strip = true           # 移除调试符号
```

*注意：由于开启了 `lto = true` 和 `codegen-units = 1`，Release 构建的时间会比平时长很多（可能需要几分钟），这是正常现象。*

## 3. 执行打包命令

在项目根目录（`blink-reminder/`）下，运行以下命令：

```bash
# 如果你之前遇到了 target 目录权限问题，请带上 CARGO_TARGET_DIR 环境变量
CARGO_TARGET_DIR=./target npx @tauri-apps/cli build
```

或者如果你全局安装了 tauri-cli：

```bash
CARGO_TARGET_DIR=./target cargo tauri build
```

## 4. 获取打包产物

打包完成后，Tauri 会自动生成对应操作系统的安装包。你可以在以下目录找到它们：

### macOS
- **独立应用 (.app)**: `target/release/bundle/macos/Blink Reminder.app`
  - *你可以直接将此文件拖入系统的「应用程序 (Applications)」文件夹中。*
- **磁盘映像 (.dmg)**: `target/release/bundle/dmg/Blink Reminder_0.1.0_aarch64.dmg` (或 x64)
  - *用于分发给其他 Mac 用户安装。*

### Windows
- **安装程序 (.msi)**: `target/release/bundle/msi/Blink Reminder_0.1.0_x64_en-US.msi`
- **独立执行文件 (.exe)**: `target/release/bundle/nsis/Blink Reminder_0.1.0_x64-setup.exe`

## 5. 常见问题

### 5.1 macOS 上的“未受信任的开发者”警告
由于我们没有使用 Apple Developer 证书对应用进行签名，其他用户在首次打开 `.dmg` 或 `.app` 时，可能会遇到 macOS 的安全拦截。
**解决方法**：
1. 在 Finder 中找到 `Blink Reminder.app`
2. 按住 `Control` 键并点击应用图标
3. 在弹出的菜单中选择“打开”
4. 在确认对话框中再次点击“打开”

### 5.2 内存占用说明
- **开发模式 (`tauri dev`)**：由于包含调试符号和热重载监听器，内存占用可能在 100MB 以上。
- **生产模式 (`tauri build`)**：经过 Release 优化后，后台常驻内存通常会稳定在 **30MB - 50MB** 左右。

## 6. 修改应用图标
如果你想更换应用的图标，请替换 `src-tauri/icons/` 目录下的文件，然后重新运行打包命令。Tauri 会自动将它们打包进最终的应用程序中。