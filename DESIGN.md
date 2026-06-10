# Blink Reminder — 眨眼休息提醒工具 设计文档

> 基于 Rust + Tauri 构建的跨平台桌面小工具，通过屏幕水波纹动画优雅地提醒用户眨眼与休息。

---

## 1. 项目概述

### 1.1 目标
提供一个**极轻量、低干扰**的后台常驻工具，在用户专注工作时，基于定时触发策略，在屏幕上展示水滴荡漾动画，提醒眨眼与休息，缓解用眼疲劳。

### 1.2 核心原则
- **轻量** — 内存占用控制在 50MB 以内，包体积 < 10MB
- **低侵入** — 提醒动画优雅、短暂，点击/按键即消失，不打断工作流
- **跨平台** — 同时支持 macOS 和 Windows
- **隐私安全** — 纯本地运行，无网络请求，无数据收集

---

## 2. 技术栈

| 层级 | 技术选型 | 说明 |
|------|---------|------|
| 桌面框架 | **Tauri v2** | 调用系统 WebView，比 Electron 轻 20 倍 |
| 后端语言 | **Rust** | 主进程：定时引擎、系统托盘、窗口管理 |
| 前端渲染 | **HTML + Canvas** | 渲染进程：水滴荡漾动画 |
| 动画引擎 | **Canvas 2D API** | 物理模拟水波纹扩散与衰减 |
| 打包分发 | **Tauri CLI** | 跨平台构建 (.dmg / .msi / .exe) |

---

## 3. 架构设计

```
┌──────────────────────────────────────────────────┐
│                   Tauri App                       │
│                                                   │
│  ┌──────────────────────────────────────────┐    │
│  │              Rust 主进程                   │    │
│  │  ┌─────────┐  ┌──────────┐  ┌────────┐  │    │
│  │  │  Tray   │  │   Timer  │  │ Window │  │    │
│  │  │  Manager│  │  Engine  │  │Manager │  │    │
│  │  └─────────┘  └──────────┘  └────────┘  │    │
│  │       │             │             │       │    │
│  │       ▼             ▼             ▼       │    │
│  │  ┌────────────────────────────────────┐   │    │
│  │  │         Tauri Commands             │   │    │
│  │  │  (Rust → JS 的 IPC 桥接层)         │   │    │
│  │  └────────────────────────────────────┘   │    │
│  └──────────────────────────────────────────┘    │
│                        │                          │
│                        ▼                          │
│  ┌──────────────────────────────────────────┐    │
│  │            WebView 渲染进程               │    │
│  │  ┌────────────────┐  ┌───────────────┐  │    │
│  │  │   Canvas 动画   │  │  设置面板 UI  │  │    │
│  │  │   (水滴波纹)     │  │  (配置界面)   │  │    │
│  │  └────────────────┘  └───────────────┘  │    │
│  └──────────────────────────────────────────┘    │
└──────────────────────────────────────────────────┘
```

### 3.1 进程模型

**主进程 (Rust)**：
- 管理系统托盘图标与菜单
- 运行定时引擎（眨眼计时器、休息计时器）
- 控制窗口生命周期（创建、显示、隐藏、关闭）
- 持久化配置（JSON 文件）

**渲染进程 (WebView)**：
- 仅在有提醒时创建/显示窗口
- 播放水滴荡漾动画
- 用户点击或按任意键后关闭窗口
- 设置面板（单独页面，通过托盘菜单打开）

---

## 4. 核心模块设计

### 4.1 定时引擎 (Timer Engine)

```rust
enum ReminderType {
    Blink,    // 眨眼提醒
    Rest,     // 休息提醒
}

struct TimerConfig {
    blink_interval: Duration,    // 默认 20 秒
    rest_interval: Duration,     // 默认 30 分钟
    work_hours: (u8, u8),       // 工作时间段，如 (9, 18)
}

struct TimerEngine {
    blink_timer: Timer,
    rest_timer: Timer,
    config: TimerConfig,
    is_paused: AtomicBool,
}
```

**触发逻辑**：
```
[系统启动] → 加载配置 → 启动计时器
    ├─ 每 20s → 触发「眨眼提醒」(小水滴，2s 后自动消失)
    ├─ 每 30min → 触发「休息提醒」(大水滴 + 文字提示，5s 后消失)
    └─ [用户暂停] → 暂停所有计时器
```

### 4.2 窗口管理器 (Window Manager)

| 窗口类型 | 属性 | 用途 |
|---------|------|------|
| 提醒窗口 | 透明、无边框、置顶、忽略鼠标事件 | 展示水滴动画 |
| 设置窗口 | 常规窗口、固定大小 400×500 | 配置参数 |

**提醒窗口生命周期**：
1. 定时器触发 → Tauri 命令调用 `show_reminder(type)`
2. 主进程创建/显示透明置顶窗口，URL 携带提醒类型参数
3. WebView 加载动画页面，播放水滴荡漾效果
4. 用户点击/按键 → 窗口关闭
5. 超时自动关闭 → 窗口隐藏

### 4.3 水滴荡漾动画 (Ripple Animation)

使用 Canvas 2D API 实现的物理模拟：

```
动画帧循环 (requestAnimationFrame):
  每帧更新：
    for each ripple in ripples:
        ripple.radius += speed * delta    // 半径扩散
        ripple.opacity -= decay * delta   // 透明度衰减
        ripple.amplitude -= damping       // 振幅衰减
        if ripple.opacity <= 0:
            remove ripple

  每帧绘制：
    for each ripple in ripples:
        绘制同心圆环（从内到外 3-5 圈）
        应用正弦波扭曲 → 产生荡漾感
        设置透明度渐变
```

**眨眼提醒动画参数**：
- 单个水波，半径从 50px → 200px
- 持续时间：~1.5 秒
- 波环数量：3 圈
- 颜色：浅蓝/透明，柔和

**休息提醒动画参数**：
- 2-3 个连续水波，半径从 80px → 400px
- 持续时间：~3 秒
- 波环数量：5 圈
- 中间显示 "该休息啦 👀" 文字
- 结束后保持小窗口提示 5 秒

### 4.4 系统托盘 (System Tray)

```
┌─────────────────────┐
│   👁 Blink Reminder  │
├─────────────────────┤
│   ▶ 开启提醒        │
│   ⏸ 暂停 20 分钟    │
│   ⏸ 暂停 1 小时     │
│   ───────────────── │
│   ⚙ 设置...         │
│   ❌ 退出            │
└─────────────────────┘
```

### 4.5 配置管理 (Config)

存储在 `~/.blink-reminder/config.json`：

```json
{
  "blink_interval_sec": 20,
  "rest_interval_min": 30,
  "blink_animation_duration_sec": 1.5,
  "rest_animation_duration_sec": 5.0,
  "ripple_color": "#4FC3F7",
  "work_start_hour": 9,
  "work_end_hour": 18,
  "enable_work_hours": true,
  "enable_sound": false,
  "theme": "light"
}
```

---

## 5. 数据流

```
用户操作                系统流程
─────────            ──────────
[启动应用]           → 初始化 Tauri App
                     → 加载配置
                     → 创建系统托盘
                     → 启动定时引擎
                     ↓
[定时器触发]         → Tauri Command: show_reminder(Blink)
                     → WindowManager 创建透明窗口
                     → 加载动画 HTML
                     ↓
[动画播放]           → Canvas 渲染水波纹
                     → 自然消散 或 用户点击
                     ↓
[窗口关闭]           → WindowManager 隐藏窗口
                     → 定时器继续
                     ↓
[用户: 打开设置]     → Tray 菜单 → Tauri Command: open_settings
                     → 创建设置窗口
                     → 用户修改配置
                     → 写入配置文件
                     → 更新定时引擎
```

---

## 6. 项目结构

```
blink-reminder/
├── src/                          # Rust 主进程代码
│   ├── main.rs                   # 入口: Tauri App 初始化
│   ├── tray.rs                   # 系统托盘管理
│   ├── timer.rs                  # 定时引擎
│   ├── window.rs                 # 窗口管理
│   ├── config.rs                 # 配置读写
│   └── commands.rs               # Tauri IPC 命令
├── src-tauri/
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   ├── capabilities/             # Tauri 权限声明
│   ├── icons/                    # 应用图标
│   └── build.rs                  # 构建脚本
├── frontend/                     # WebView 前端
│   ├── index.html                # 动画页面入口
│   ├── settings.html             # 设置页面入口
│   ├── styles/
│   │   └── main.css              # 全局样式
│   └── scripts/
│       ├── ripple.js             # 水波纹动画引擎
│       └── settings.js           # 设置页面逻辑
├── DESIGN.md                     # 本设计文档
└── README.md                     # 使用说明
```

---

## 7. 实现计划

### 阶段一：原型 (1-2 天)
- [x] 完成设计文档
- [ ] 初始化 Tauri 项目
- [ ] 实现系统托盘（带基本菜单）
- [ ] 实现透明置顶窗口（点击关闭）
- [ ] 实现 Canvas 水波纹动画
- [ ] 实现基本定时引擎（眨眼 20s / 休息 30min）

### 阶段二：完善 (1-2 天)
- [ ] 实现设置面板（UI + 配置持久化）
- [ ] 添加工作时间段判断逻辑
- [ ] 添加暂停功能
- [ ] 完善动画效果（多波、颜色、平滑度）

### 阶段三：优化与分发 (1 天)
- [ ] Windows 兼容性测试
- [ ] 应用图标
- [ ] 打包配置（.dmg + .msi）
- [ ] 开机自启功能
- [ ] README 编写

---

## 8. 未解决问题 / 后续可扩展

- **摄像头检测** — 可选功能：通过摄像头检测是否真的在眨眼，不眨眼才提醒（使用 OpenCV）
- **专注模式集成** — 与系统勿扰模式联动
- **Pomodoro 番茄钟** — 内置番茄钟工作法
- **自定义动画** — 允许用户上传自定义提醒动画
- **统计看板** — 记录每日眨眼/休息频率，生成简单统计
- **快捷键** — 全局快捷键快速暂停/恢复

---

## 9. 附录

### 9.1 依赖清单 (Cargo.toml)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["time"] }
tray-icon = "0.3"

[build-dependencies]
tauri-build = "2"
```

### 9.2 性能目标

| 指标 | 目标 |
|------|------|
| 空闲内存占用 | < 40 MB |
| 提醒动画 CPU 占用 | < 5% (单核) |
| 启动时间 | < 1 秒 |
| 包体积 (macOS) | < 10 MB |
| 包体积 (Windows) | < 8 MB |