# bitclk

`bitclk` 是一个偏 CLI / hacker / minimal / geek 风格的终端时间工具。  
第一版先把 `clock` 做扎实：默认以二进制时钟启动，同时支持在运行时切换到普通数字时钟。

## 当前已实现

- `bitclk` 默认进入 `clock`
- `bitclk clock`
- `bitclk clock --binary`
- `bitclk clock --normal`
- 运行时在 normal / binary 两种视图之间切换
- 使用本地时间，每秒至少刷新一次
- 使用 alternate screen + raw mode，并在退出时恢复终端状态
- `bitclk stopwatch`
- `bitclk timer`

说明：
`stopwatch` 和 `timer` 当前还是脚手架，占位命令会输出友好的 `not implemented yet` 提示，但整个项目可正常编译运行。

## 运行方式

```bash
cargo run
```

```bash
cargo run -- clock
```

```bash
cargo run -- clock --normal
```

```bash
cargo run -- clock --binary
```

## 运行时快捷键

- `q`：退出
- `tab`：在 binary / normal 间切换
- `b`：切到 binary clock
- `n`：切到 normal clock

## 目录结构

```text
src/
├── app.rs
├── cli.rs
├── main.rs
├── terminal.rs
├── modes/
│   ├── clock.rs
│   ├── mod.rs
│   ├── stopwatch.rs
│   └── timer.rs
└── render/
    ├── binary_clock.rs
    ├── mod.rs
    └── normal_clock.rs
```

## 模块组织

- `cli.rs`
  使用 `clap` 定义根命令、子命令和 `clock` 的启动模式参数。
- `app.rs`
  负责把 CLI 输入分发到具体模式，根命令无子命令时默认解析为 `clock`。
- `modes/`
  放各个运行模式。当前 `clock` 完整可用，`stopwatch` / `timer` 为可扩展占位模块。
- `render/`
  放时钟渲染逻辑。普通时钟和二进制时钟分离实现，共享基础布局和屏幕组合逻辑。
- `terminal.rs`
  封装终端会话的进入与恢复，保证退出时尽量不留下坏掉的终端状态。

## 已实现与预留扩展

已实现：

- 可运行的 `clock` 模式
- normal / binary 双视图
- 运行时快捷键切换
- 二进制 `HH / MM / SS` 直出渲染
- 占位子命令和基础项目架构

为未来预留：

- `stopwatch` / `timer` 的正式实现
- 更多 binary styles，例如 dots / block / dense cells
- teach / explanation 模式
- 位变化高亮、pulse、breathing 等动画
- 更丰富的 CLI 参数，例如 `--style`、`--mode`

## 开发检查

```bash
cargo fmt
cargo check
cargo test
```
