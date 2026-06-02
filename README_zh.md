<p align="center">
    <img src="docs/assets/logo-large.png" alt="rkiauh Logo" height="181">
    <h1 align="center">rkiauh: 基于 Rust 的 Klipper 安装与更新助手</h1>
</p>

<p align="center">
  一个基于 Rust 重写的高性能、编译型系统部署工具，替代了传统的 shell 脚本版本。目标平台: <b>MKS SKIPR (运行 Armbian Linux 的 Cortex-A53 RK3328 架构板卡)</b>。
</p>

<p align="center">
  <a><img src="https://img.shields.io/github/license/dw-0/kiauh"></a>
  <a><img src="https://img.shields.io/github/stars/dw-0/kiauh"></a>
  <a><img src="https://img.shields.io/github/languages/top/dw-0/kiauh?logo=rust&logoColor=white"></a>
</p>

<hr>

## 🚀 核心特性

* **终端交互式 TUI**：基于 `ratatui` (0.30.0) 和 `crossterm` (0.29.0) 构建的高性能控制台，支持服务状态的实时监视、编译日志流式显示以及动态的交互式按键指令。
* **原生的 Systemd 控制**：弃用低效的 `systemctl` 命令行子进程调用，通过原生 D-Bus 系统总线接口 (`org.freedesktop.systemd1`) 通信，依托 `zbus` (5.16.0) 库实现高效稳健的进程管控。
* **纯 Rust Git 仓库管理**：内置纯 Rust 的 `git2` (0.21.0) 接口，直接在二进制中操作源码的克隆（Clone）、拉取（Pull）和版本状态更新。
* **动态配置模板引擎**：集成编译期 Tera 模板系统，支持动态填入系统级负载变量（如 Moonraker 端口、Nginx 上游及监听端口）自动生成 Nginx 反向代理配置，并安全写入磁盘。

---

## 🛠️ 受管控组件列表

本工具全面跟踪并管理以下五个核心打印机组件：

| 组件名称 | 托管 Git 源码仓库 | 本地工作区路径 | 控制类型 |
|---|---|---|---|
| **r_klipp** | [FaezBarghasa/r_klipp](https://github.com/FaezBarghasa/r_klipp) | `.../r_klipp` | 系统服务守护进程 (`r_klipp.service`) |
| **rusted_moonraker** | [FaezBarghasa/rusted_moonraker](https://github.com/FaezBarghasa/rusted_moonraker) | `.../rusted_moonraker` | 系统服务守护进程 (`rusted_moonraker.service`) |
| **rKlipperScreen** | [FaezBarghasa/rKlipperScreen](https://github.com/FaezBarghasa/rKlipperScreen) | `.../rKlipperScreen` | 系统服务守护进程 (`rKlipperScreen.service`) |
| **fluidd** | [fluidd-core/fluidd](https://github.com/fluidd-core/fluidd) | `.../kiauh/docs` | 静态网页客户端面板 |
| **mainsail** | [mainsail-crew/mainsail](https://github.com/mainsail-crew/mainsail) | `.../mainsail` | 静态网页客户端面板 |

---

## 🎮 TUI 控制键位与指南

通过底部的快捷导引栏可执行如下操作：

* `[↑ / ↓]` 或 `[k / j]` - 在服务监视表单中上下选择组件。
* `[i]` - **安装**当前选中的组件（执行 Git 源码拉取，通过 Cargo 自动化调用编译，写入 systemd service 配置文件）。
* `[u]` - **更新**选定组件（获取最新的远程更新，重新进行编译部署，并平滑重启对应守护进程）。
* `[s]` - **启动 (Start)** 选中组件的 systemd 服务。
* `[t]` - **停止 (Stop)** 选中组件的 systemd 服务。
* `[r]` - **重启 (Restart)** 选中组件的 systemd 服务。
* `[c]` - 调出 **Nginx 反向代理配置向导**，输入端口、主机名及静态目录变量一键生成与应用。
* `[q]` 或 `[Esc]` - 安全退出。

---

## 💻 编译与运行方式

1. 编译生成 Release 版本工具：
   ```bash
   cargo build --release
   ```
2. 直接启动编译后的 TUI 应用程序：
   ```bash
   ./target/release/rkiauh
   ```
3. 运行内置的测试用例验证模板渲染正确性：
   ```bash
   cargo test
   ```
