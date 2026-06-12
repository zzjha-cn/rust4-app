# Blink Reminder

rust101

Blink Reminder 是一款轻量级的桌面护眼提醒工具。它常驻系统托盘，在设定的时间间隔内，通过在屏幕上播放透明的水波纹（涟漪）动画，提醒用户眨眼或休息，从而缓解长时间盯屏幕带来的视觉疲劳。

本项目在探索过程中，为了追求最佳的性能与用户体验，演进出了**两套不同的技术实现方案**：基于 Tauri 的 WebView 方案，以及基于纯 Rust 的系统原生方案。

---

## 方案一：纯原生实现 (Native Implementation)
**目录**：`/blink-reminder-native`

这是目前项目的**主推方向**，旨在解决 WebView 方案带来的高内存占用问题，追求极致的性能和极低的系统资源消耗。

### 技术栈
- **核心逻辑**：Rust, `tokio` (定时器调度), `tao` (事件循环), `tray-icon` (系统托盘)
- **macOS 渲染层**：`objc2`, `AppKit`, `CoreImage`, `CoreGraphics`
- **Windows 渲染层**：`windows-rs`, `Direct2D` (规划中)

### 实现原理
1. **分层架构**：将核心业务逻辑（定时、配置、托盘）与底层渲染（macOS/Windows）完全解耦。
2. **macOS 渲染机制**：
   - 创建 `NSWindowStyleMask::Borderless` 的透明无边框窗口，层级设置为 Overlay。
   - 动画触发时，通过 `CGDisplay::main().image()` 瞬间截取当前屏幕画面。
   - 将截图作为 `CALayer` 的内容，并应用 `CITorusLensDistortion` (环面透镜畸变) 滤镜。
   - 使用 `CABasicAnimation` 动态改变滤镜的 `inputRadius`，实现逼真的水波纹折射与放大效果。
3. **双播机制**：单次提醒内，通过主线程事件循环连续触发两次波纹动画（间隔 0.5 秒）。

### 优缺点
- **优点**：极致的性能，极低的内存占用（常驻内存极小，动画执行期间也不会有明显的内存飙升），真正的系统级无感提醒。
- **缺点**：需要针对不同操作系统（macOS, Windows）分别编写底层图形 API 的调用代码，开发和调试成本较高。

### 编译与运行
```bash
cd blink-reminder-native
cargo run --release
```

---

## 方案二：Tauri + WebView 实现 (Tauri Implementation)
**目录**：项目根目录 (`/src-tauri`, `/frontend`)

这是项目初期的实现方案，利用 Web 技术栈快速构建了跨平台的动画效果。

### 技术栈
- **后端**：Rust, Tauri v2
- **前端**：HTML, CSS, JavaScript (Vanilla)

### 实现原理
1. **窗口管理**：Tauri 后端创建一个全屏、透明、无边框、忽略鼠标事件（`set_ignore_cursor_events`）的窗口。
2. **动画渲染**：
   - 前端通过 CSS 的 `backdrop-filter: blur()` 和自定义的 `mask-image` (径向渐变) 来模拟水波纹的折射和放大效果。
   - 使用 JavaScript 动态生成 DOM 元素（波纹圈），并通过 CSS `@keyframes` 控制其缩放和透明度变化。
3. **生命周期优化**：为了降低常驻内存，动画播放完毕后会调用 `window.close()` 销毁窗口，下次提醒时再重新创建。

### 优缺点
- **优点**：开发效率高，CSS 动画调试直观，天然跨平台（macOS/Windows 均可直接运行）。
- **缺点**：由于全屏透明窗口和复杂的 CSS `backdrop-filter` 渲染，动画执行时会引发极高的内存占用（峰值可达 2GB），即使优化后，对于一个轻量级后台工具来说依然偏重。

### 编译与运行
```bash
# 安装前端依赖
npm install

# 启动开发环境
npm run tauri dev

# 构建生产版本
npm run tauri build
```

---

## 核心特性 (通用)

无论采用哪种方案，Blink Reminder 都具备以下核心特性：

- **无感提醒**：动画直接作用于当前屏幕内容，不抢占键盘焦点。
- **智能专注模式**：彻底摒弃死板的定时提醒。将检查周期划分为多个时间窗口，只有当用户在周期内的活跃窗口数量达到设定阈值（即真正处于高强度专注状态）时，才会触发提醒。如果用户在发呆或看视频，则自动跳过提醒。
- **智能防打扰 (输入延迟)**：底层监听系统级空闲状态。当判定需要提醒时，如果用户正在活跃打字或操作鼠标（空闲时间 < 1秒），动画会自动推迟，直到用户停下输入超过 1 秒后才会温柔出现，绝不打断工作流。
- **动态配置与丰富托盘**：支持通过托盘菜单实时开启/关闭提醒、快速切换提醒间隔、测试动画效果，配置修改即时生效，无需重启应用。
- **工作时间控制**：可配置仅在设定的工作时间段内（如 9:00 - 22:00）触发提醒。

## 配置文件

配置文件统一保存在用户目录下的 `~/.blink-reminder/config.json`。

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

## 后续计划

- [ ] **Native 方案**：完成 Windows 平台的 Direct2D 渲染实现。
- [ ] **Native 方案**：完善长时间休息（Rest）的动画与交互逻辑。
- [ ] **通用**：提供一个轻量级的可视化设置界面。
