<p align="center">
    <img src="docs/assets/logo-large.png" alt="rkiauh Logo" height="181">
    <h1 align="center">rkiauh: Rust-based Klipper Installation & Update Helper</h1>
</p>

<p align="center">
  A high-performance, compiled Rust system provisioner utility rewritten from the legacy shell-based KIAUH helper. Target platform: <b>MKS SKIPR (Cortex-A53 RK3328 running Armbian Linux)</b>.
</p>

<p align="center">
  <a><img src="https://img.shields.io/github/license/dw-0/kiauh"></a>
  <a><img src="https://img.shields.io/github/stars/dw-0/kiauh"></a>
  <a><img src="https://img.shields.io/github/languages/top/dw-0/kiauh?logo=rust&logoColor=white"></a>
</p>

<hr>

## 🚀 Key Features

* **Terminal Alternate-Screen TUI**: Real-time service monitoring, live compilation log console, and dynamic interactive commands powered by `ratatui` (0.30.0) and `crossterm` (0.29.0).
* **Native Systemd Control**: Bypasses systemctl CLI execution hacks by communicating directly over the Linux D-Bus IPC system bus (`org.freedesktop.systemd1`) using the pure-Rust `zbus` (5.16.0) crate.
* **Native Git Management**: Programmatically clones, pulls, and updates codebase configurations using pure-Rust `git2` (0.21.0) library bindings.
* **Dynamic Configuration Templating**: Generates customized deployment environments (e.g. Nginx reverse-proxies) on the fly via the compiled-in `tera` template parser and writes them safely to disk.

---

## 🛠️ Monitored Components

The utility manages and monitors five core print-server components:

| Component | Target Git Repository | Local workspace path | Type |
|---|---|---|---|
| **r_klipp** | [FaezBarghasa/r_klipp](https://github.com/FaezBarghasa/r_klipp) | `.../r_klipp` | Service Daemon (`r_klipp.service`) |
| **rusted_moonraker** | [FaezBarghasa/rusted_moonraker](https://github.com/FaezBarghasa/rusted_moonraker) | `.../rusted_moonraker` | Service Daemon (`rusted_moonraker.service`) |
| **rKlipperScreen** | [FaezBarghasa/rKlipperScreen](https://github.com/FaezBarghasa/rKlipperScreen) | `.../rKlipperScreen` | Service Daemon (`rKlipperScreen.service`) |
| **fluidd** | [fluidd-core/fluidd](https://github.com/fluidd-core/fluidd) | `.../kiauh/docs` | Static Client Web Interface |
| **mainsail** | [mainsail-crew/mainsail](https://github.com/mainsail-crew/mainsail) | `.../mainsail` | Static Client Web Interface |

---

## 🎮 TUI Controls & Shortcuts

Navigate using the keyboard guides in the footer guide bar:

* `[↑ / ↓]` or `[k / j]` - Select a printer component in the main table.
* `[i]` - **Install** the selected component (Git clones, programmatically compiles using Cargo cross-compilation loops, and generates systemd unit files).
* `[u]` - **Update** the selected component (performs pull fetch, re-compiles, and restarts service daemons).
* `[s]` - **Start** the systemd service for the selected component.
* `[t]` - **Stop** the systemd service for the selected component.
* `[r]` - **Restart** the systemd service for the selected component.
* `[c]` - Launch the interactive **Nginx configuration wizard** (prompts for Moonraker port, Nginx listen port, server hostnames, and static pathing).
* `[q]` or `[Esc]` - Safe exit.

---

## 💻 Compilation and Execution

1. Build the utility locally in release mode:
   ```bash
   cargo build --release
   ```
2. Execute the TUI binary directly:
   ```bash
   ./target/release/rkiauh
   ```
3. Run the unit test suite verifying template rendering correctness:
   ```bash
   cargo test
   ```
