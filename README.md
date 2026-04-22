# bitclk

`bitclk` 是一个偏 CLI / hacker / minimal / geek 风格的终端时间工具。  
`clock`、`stopwatch`、`timer` 默认都以二进制钟表启动，主界面只显示钟表本体；可在运行时切换为八进制或十六进制，并切换横向 / 竖向排布。

## 当前已实现

- `bitclk` 默认进入 `clock`
- `bitclk clock`
- `bitclk clock --binary`
- `bitclk clock --octal`
- `bitclk clock --hex`
- `bitclk --transparent`
- `clock` / `stopwatch` / `timer` 默认以二进制显示
- 运行时在 binary / octal / hexadecimal 间切换
- 运行时切换横向 / 竖向排布
- 运行时按 `h` 打开帮助面板
- 使用本地时间，每秒至少刷新一次
- 使用 alternate screen + raw mode，并在退出时恢复终端状态
- `bitclk stopwatch`
- `bitclk timer`
- `bitclk timer 05:00`
- `bitclk --theme "#3b82f6"`
- `bitclk --theme "#3b82f6" --mode analogous timer 05:00`
- `bitclk theme "#3b82f6" --mode triadic`
- 配色引擎（HSL hue rotation）
- 内置 15 组默认主题预设
- 基础主题系统（primary / secondary / accent / background / foreground / muted）
- 运行时循环切换主题预设

说明：
`clock`、`stopwatch`、`timer` 现在都只在主界面显示钟表本体。`timer` 支持可选初始时长参数，也支持进入后用方向键调整倒计时；`stopwatch` 和 `timer` 的小时上限都是 `63`。运行时 `--theme` 和 `theme` 子命令不会冲突：`--theme` 是直接带着生成出来的主题运行钟表，`theme` 子命令只是预览配色结果。

## 运行方式

```bash
cargo run
```

```bash
cargo run -- clock
```

```bash
cargo run -- clock --octal
```

```bash
cargo run -- clock --hex
```

```bash
cargo run -- --transparent
```

```bash
cargo run -- timer 05:00
```

```bash
cargo run -- --theme "#3b82f6"
```

```bash
cargo run -- --theme "#3b82f6" --mode analogous timer 05:00
```

```bash
cargo run -- theme "#3b82f6" --mode triadic
```

```bash
cargo run -- --help
```

## 运行时快捷键

- `clock`
- `q`：退出
- `b / o / x`：切换 binary / octal / hexadecimal
- `tab`：切换横向 / 竖向排布
- `t`：默认循环切换内置主题预设；如果启动时带了 `--theme`，则循环切换 harmony mode
- `h`：显示 / 关闭帮助
- `stopwatch`
- `space`：开始 / 暂停
- `r`：重置
- `b / o / x`：切换 binary / octal / hexadecimal
- `tab`：切换横向 / 竖向排布
- `t`：默认循环切换内置主题预设；如果启动时带了 `--theme`，则循环切换 harmony mode
- `h`：显示 / 关闭帮助
- `q`：退出
- `timer`
- `space`：开始 / 暂停
- `↑ / ↓`：加减 1 分钟
- `← / →`：加减 10 秒
- `PgUp / PgDn`：加减 10 分钟
- `0`：清零
- `r`：重置到当前设定时长
- `b / o / x`：切换 binary / octal / hexadecimal
- `tab`：切换横向 / 竖向排布
- `t`：默认循环切换内置主题预设；如果启动时带了 `--theme`，则循环切换 harmony mode
- `h`：显示 / 关闭帮助
- `q`：退出

## 配色说明

- `--theme "#3b82f6"`：把这个 HEX 颜色当作基础色，交给配色引擎生成整套主题后直接运行。
- `bitclk theme "#3b82f6"`：只预览这套主题，不启动时钟。
- `--mode triadic`：使用 triadic harmony，意思是以基础色为起点，在色环上取约 120 度间隔的三组主色，所以整体会更均衡、更有三色对比感。
- 当前支持的 harmony mode：`complementary`、`analogous`、`triadic`、`split-complementary`。

## 目录结构

```text
src/
├── color.rs
├── color_engine.rs
├── app.rs
├── cli.rs
├── main.rs
├── terminal.rs
├── theme.rs
├── modes/
│   ├── clock.rs
│   ├── mod.rs
│   ├── stopwatch.rs
│   ├── theme_demo.rs
│   └── timer.rs
└── render/
    ├── brick_text.rs
    ├── binary_clock.rs
    └── mod.rs
```

## 模块组织

- `cli.rs`
  使用 `clap` 定义根命令、子命令、`clock` 的启动进制参数，以及 `timer` 的可选倒计时时长。
- `app.rs`
  负责把 CLI 输入分发到具体模式，根命令无子命令时默认解析为 `clock`。
- `color.rs`
  放 RGB / HSL 数据结构、HEX 转换、亮度与对比度处理，以及终端颜色适配辅助函数。
- `color_engine.rs`
  放配色模式（complementary / analogous / triadic / split-complementary）与主题生成策略。
- `theme.rs`
  定义可复用的 `Theme` 结构，统一管理主色、辅助色、强调色以及前景/背景语义色。
- `modes/`
  放各个运行模式。当前 `clock` / `stopwatch` / `timer` 都可直接使用，`theme_demo` 用于预览主题。
- `render/`
  放钟表渲染逻辑。当前统一支持二进制 / 八进制 / 十六进制，以及横向 / 竖向排布，共享基础布局和屏幕组合逻辑，并消费统一的 `Theme`。
- `terminal.rs`
  封装终端会话的进入与恢复，保证退出时尽量不留下坏掉的终端状态。

## 已实现与预留扩展

已实现：

- 可运行的 `clock` 模式
- binary / octal / hexadecimal 三种钟表模式
- 横向 / 竖向排布切换
- 运行时快捷键帮助面板
- 二进制 6 位 `HH / MM / SS` 渲染
- 八进制 / 十六进制 2 位 `HH / MM / SS` 渲染
- 可运行的 `stopwatch`
- 可运行的 `timer`
- `timer` 可选初始时长参数
- `stopwatch` / `timer` 小时数上限 `63`
- `--transparent` 透明背景画布
- 基于基础色自动生成 CLI 主题
- 内置 15 组运行时主题预设
- foreground / accent 自动做基础可读性修正
- 可扩展的多模式架构

为未来预留：

- 更多 binary styles，例如 dots / block / dense cells
- 通过统一 `Theme` 接到 `crossterm` / `ratatui`
- teach / explanation 模式
- 位变化高亮、pulse、breathing 等动画
- 更丰富的 CLI 参数，例如 `--style`、`--mode`

## 开发检查

```bash
cargo fmt
cargo check
cargo test
```
