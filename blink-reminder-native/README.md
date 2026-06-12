# Blink Reminder

Blink Reminder 是一款轻量级的桌面护眼提醒工具。它常驻系统托盘，在设定的时间间隔内，通过在屏幕上播放透明的水波纹（涟漪）动画，提醒用户眨眼或休息，从而缓解长时间盯屏幕带来的视觉疲劳。

该项目采用 Rust 编写，追求极低的内存占用和原生级别的渲染性能。

## 核心特性

- **极低资源占用**：采用 Rust 原生实现，摒弃了 WebView 方案，常驻内存极低。
- **无感提醒**：动画效果为透明的水波纹折射（类似放大镜效果），直接作用于当前屏幕内容，不抢占键盘焦点。
- **智能专注模式**：彻底摒弃死板的定时提醒。将检查周期划分为多个时间窗口，只有当用户在周期内的活跃窗口数量达到设定阈值（即真正处于高强度专注状态）时，才会触发提醒。如果用户在发呆或看视频，则自动跳过提醒。
- **智能防打扰 (输入延迟)**：底层监听系统级空闲状态（如 macOS 的 `CoreGraphics` API）。当判定需要提醒时，如果用户正在活跃打字或操作鼠标（空闲时间 < 1秒），动画会自动推迟，直到用户停下输入超过 1 秒后才会温柔出现，绝不打断工作流。
- **双播机制**：单次提醒内连续播放两次波纹（间隔 0.5 秒），确保提醒效果。
- **动态配置与丰富托盘**：支持通过托盘菜单实时开启/关闭提醒、快速切换提醒间隔、测试动画效果，配置修改即时生效，无需重启。
- **跨平台架构**：核心逻辑与渲染层解耦，目前已实现 macOS 原生渲染，并为 Windows 平台预留了 Direct2D 渲染接口。

## 架构设计

项目采用了清晰的分层架构，以便于跨平台扩展：

1. **核心逻辑层 (`src/main.rs`, `src/timer.rs`, `src/config.rs`)**
   - 负责托盘图标管理（基于 `tray-icon`）。
   - 负责定时器调度（基于 `tokio`），支持动态读取配置。
   - 负责配置的持久化与热更新。

2. **渲染抽象层 (`src/render/mod.rs`)**
   - 定义了 `RippleRenderer` trait，统一了不同平台的渲染接口：
     - `setup()`: 初始化渲染环境。
     - `show_ripple()`: 播放波纹动画。
     - `hide_ripple()`: 隐藏动画并清理资源。

3. **macOS 实现层 (`src/render/mac.rs`)**
   - 使用 `objc2` 和相关框架调用 macOS 原生 API。
   - 创建 `NSWindowStyleMask::Borderless` 的透明无边框窗口。
   - 通过 `CGDisplay::main().image()` 截取当前屏幕，并应用 `CITorusLensDistortion` 滤镜实现水波纹折射动画。

4. **Windows 实现层 (`src/render/win.rs`)**
   - 预留了基于 `windows-rs` 的实现框架。
   - 计划使用 `WS_EX_LAYERED | WS_EX_TRANSPARENT` 窗口，结合 Direct2D / DirectComposition 进行渲染。

## 配置文件

配置文件默认保存在用户目录下的 `~/.blink-reminder/config.json`。

```json
{
  "blink_interval_sec": 40,
  "time_window_sec": 3,
  "active_window_threshold": 10,
  "rest_interval_min": 40,
  "blink_animation_duration_sec": 1.2,
  "rest_animation_duration_sec": 5.0,
  "ripple_color": "#4FC3F7",
  "work_start_hour": 9,
  "work_end_hour": 22,
  "enable_work_hours": true,
  "enable_sound": false,
  "enable_blink": true,
  "theme": "light"
}
```

- `blink_interval_sec`: 眨眼提醒的检查周期（秒）。
- `time_window_sec`: 智能专注模式的时间窗口大小（秒）。
- `active_window_threshold`: 触发提醒所需的活跃窗口数量阈值。
- `blink_animation_duration_sec`: 单次波纹动画的持续时间（秒）。
- `enable_blink`: 是否开启眨眼提醒。
- `work_start_hour` / `work_end_hour`: 工作时间段，仅在该时间段内触发提醒。

## 编译与运行

### 环境要求

- Rust (最新稳定版)
- macOS (当前仅 macOS 渲染层已完全实现)

### 运行

```bash
cd blink-reminder-native
cargo run --release
```

### 构建

```bash
cd blink-reminder-native
cargo build --release
```
编译后的可执行文件位于 `target/release/blink-reminder-native`。

## 后续计划

- [ ] 完成 Windows 平台的 Direct2D 渲染实现。
- [ ] 完善长时间休息（Rest）的动画与交互逻辑。
- [ ] 提供一个轻量级的设置界面（可选）。
