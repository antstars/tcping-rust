

# tcping-rust


[🇨🇳 简体中文](#简体中文) | [🇬🇧 English](#english)

A lightweight, fast, and cross-platform TCP ping tool written in Rust. It measures network latency by establishing TCP connections, making it an excellent alternative to traditional ICMP ping, especially when ICMP is blocked by firewalls.

## Features

* **Blazing Fast & Lightweight**: Written in Rust, compiled to a single native binary with zero runtime dependencies.
* **Cross-Platform**: Natively supports Windows, macOS, and Linux (both AMD64 and ARM64 architectures).
* **Precise Statistics**: Accurately calculates RTT (Round-Trip Time), total uptime, downtime, and packet loss.
* **Continuous Monitoring**: Supports continuous pinging (like `ping -t`) with graceful Ctrl+C interruption and final reporting.

## Installation

### Method 1: Download Pre-compiled Binaries (Recommended)

Go to the [Releases](../../releases) page and download the executable file that matches your operating system and architecture.

* Linux: `tcping-linux-amd64` / `tcping-linux-arm64`
* Windows: `tcping-windows-amd64.exe` / `tcping-windows-arm64.exe`
* macOS: `tcping-macos-intel` / `tcping-macos-arm64`

### Method 2: Build from Source

If you have the Rust toolchain installed, you can easily build it yourself:

```bash
git clone [https://github.com/YOUR_USERNAME/tcping-rust.git](https://github.com/YOUR_USERNAME/tcping-rust.git)
cd tcping-rust
cargo build --release
```

The compiled binary will be located at `target/release/tcping`.

## Usage

Basic syntax:

**Bash**

```
tcping <HOST> <PORT> [OPTIONS]
```

**Examples:**

1. **Ping a specific port 4 times (Default):**
   **Bash**

   ```
   tcping 1.1.1.1 443
   ```
2. **Ping continuously until stopped (Ctrl+C):**
   **Bash**

   ```
   tcping dns.google 853 -t
   ```
3. **Specify the number of pings:**
   **Bash**

   ```
   tcping github.com 22 -c 10
   ```
4. **Set a custom timeout (e.g., 500ms):**
   **Bash**

   ```
   tcping 8.8.8.8 53 -w 500
   ```

Use `tcping --help` to see all available options.

---

<h2 id="简体中文">简体中文</h2>

一个使用 Rust 编写的轻量、极速且跨平台的 TCP Ping 工具。它通过建立 TCP 连接来测量网络延迟，是传统 ICMP ping 的绝佳替代方案，尤其适用于 ICMP 被防火墙拦截的网络环境。

## 核心特性

* **极速且轻量** ：基于 Rust 构建，编译为无任何运行时依赖的单一机器码文件。
* **全平台覆盖** ：原生支持 Windows、macOS 和 Linux（涵盖 AMD64 与 ARM64 架构）。
* **精准统计** ：精确计算 RTT 延迟、总连通时间、总停机时间以及丢包率。
* **持续监测** ：支持连续 Ping（类似 `ping -t`），并在按下 `Ctrl+C` 优雅中断时输出最终统计报表。

## 安装指南

### 方式一：下载预编译版本（推荐）

前往 [Releases](https://www.google.com/search?q=../../releases&authuser=2) 页面，下载与你的操作系统和架构相匹配的可执行文件即可开箱即用。

### 方式二：源码编译

如果你已安装 Rust 工具链，可直接在本地编译：

**Bash**

```
git clone [https://github.com/你的用户名/tcping-rust.git](https://github.com/你的用户名/tcping-rust.git)
cd tcping-rust
cargo build --release
```

编译产物将生成在 `target/release/tcping` 目录下。

## 使用方法

基础语法：

**Bash**

```
tcping <主机名或IP> <端口> [选项]
```

**操作示例：**

1. **测试指定端口 4 次（默认行为）：**
   **Bash**

   ```
   tcping 1.1.1.1 443
   ```
2. **连续不间断测试，直到手动停止 (Ctrl+C)：**
   **Bash**

   ```
   tcping dns.google 853 -t
   ```
3. **指定测试次数：**
   **Bash**

   ```
   tcping github.com 22 -c 10
   ```
4. **自定义超时时间（例如 500 毫秒）：**
   **Bash**

   ```
   tcping 8.8.8.8 53 -w 500
   ```

你可以通过运行 `tcping --help` 来查看所有的命令行参数帮助。
