## nicman - Linux网络接口管理工具

### ✅ 已实现功能

#### 核心功能
- ✅ **接口列表显示**：自动识别10种接口类型（物理、Docker、TUN/TAP、WireGuard、Bridge、Veth、VLAN等）
- ✅ **实时流量监控**：显示上下行流量速率、累计流量、包数统计
- ✅ **创建者检测**：自动检测接口的创建者（systemd服务、Docker容器、进程、NetworkManager等）
- ✅ **智能删除**：根据创建者类型自动选择删除策略，安全删除虚拟接口
- ✅ **IP配置编辑**：编辑物理接口的IP地址、子网掩码、网关、DNS
- ✅ **DHCP/静态切换**：一键切换物理接口的DHCP和静态IP模式
- ✅ **接口启用/禁用**：支持接口的up/down操作
- ✅ **Netplan集成**：自动修改Netplan配置并应用，支持配置备份

#### 技术特性
- ✅ **运行时修改**：使用`ip`命令立即生效
- ✅ **持久化配置**：修改Netplan YAML文件，重启后保持
- ✅ **安全保护**：检测SSH接口和默认路由，防止误操作
- ✅ **TUI界面**：美观的终端图形界面，支持键盘导航

### 快捷键说明

#### 导航
- `↑` / `k` - 上移
- `↓` / `j` - 下移

#### 物理接口操作
- `e` - 编辑IP/掩码/网关/DNS
- `t` - 切换DHCP/静态模式
- `u` - 启用接口 (Up)
- `d` - 禁用接口 (Down)

#### 虚拟接口操作
- `x` / `Delete` - 删除接口
- `u` - 启用接口 (Up)
- `d` - 禁用接口 (Down)

#### 通用操作
- `r` - 刷新接口列表
- `q` - 退出程序
- `?` - 显示/隐藏帮助

#### 编辑表单
- `Tab` - 下一个字段
- `Shift+Tab` - 上一个字段
- `Enter` - 保存配置
- `Esc` - 取消编辑

### 使用方法

```bash
# 编译
cargo build --release

# 运行（需要root权限）
sudo ./target/release/nicman

# 或者安装到系统
sudo cp target/release/nicman /usr/local/bin/
sudo nicman
```

### 界面布局

```
┌─ 网络接口列表 ─────────┐┌─ 接口详情 ─────────────┐
│ 🔌 ✅ enp4s0           ││ 接口名称: enp4s0        │
│ 🔌 ✅ eno1             ││ 类型: Physical          │
│ 🐳 ✅ docker0          ││ 状态: Up                │
│ 🚇 ✅ tun0             ││ IPv4: 192.168.1.100/24  │
│ ...                    ││ 网关: 192.168.1.1       │
└────────────────────────┘└─────────────────────────┘
                          ┌─ 流量统计 ─────────────┐
                          │ 接收: 1.2 GB (1.5M包)  │
                          │ 发送: 800 MB (1.2M包)  │
                          │ 速率: ↓5.2MB/s ↑2.1MB/s│
                          └─────────────────────────┘
```

---

## 给 Claude 用的提示词（Rust + TUI + 网卡管理）

下面这段你可以直接复制给 Claude，用来生成完整项目骨架和主要代码逻辑：

````markdown
你现在是一名 **资深 Rust 系统开发工程师 + Linux 网络工程师 + TUI UI 设计师**，目标环境是 **Ubuntu 20.04 / 22.04 / 24.04**。

请帮我实现一个 **终端图形界面（TUI）的网卡管理小工具**，使用 **Rust** 编写，具备如下能力：

- 管理所有网络接口（物理 + 虚拟）
- 删除虚拟接口（tun/tap、WireGuard、docker0、bridge、veth、VLAN 等）
- 管理物理网卡 IP / 子网掩码 / 网关 / DNS，支持静态和 DHCP
- 支持运行时修改（立刻生效）+ 持久修改（修改 Netplan 配置并 apply）
- 在终端中以「图形界面」方式展示（使用 ratatui / tui-rs 风格界面）

---

## 一、技术栈要求

1. 语言：**Rust**（稳定版）
2. 依赖（推荐）：
   - `ratatui`（或最新的 `tui-rs` 分支，用于 TUI 绘制）
   - `crossterm`（终端事件和输入处理）
   - `clap`（解析命令行参数）
   - `serde` + `serde_yaml`（解析和写入 Netplan YAML）
   - `nix`（检查 root / 发送信号等）
   - 如需执行系统命令，使用 `std::process::Command`
3. 输出形式：
   - 一个可以 `cargo run` 的完整项目
   - 入口文件 `src/main.rs`
   - 可以按模块拆分（例如 `ui.rs`, `backend/netplan.rs`, `backend/runtime.rs`, `model.rs`）

---

## 二、功能总览

### 1. 支持管理的接口类型

- **物理接口**：`eth0`, `ens33`, `enp3s0` 等
- **虚拟接口**：
  - Tun/Tap：`tun0`, `tap0`, `vpn0` 等
  - WireGuard：`wg0`, `wg1` 等
  - Bridge：`br0`, `docker0` 等
  - 容器 veth：`vethXXXXX`
  - VLAN：`eth0.10`, `enp3s0.100` 等
- **Loopback**：`lo`（显示，但默认禁止删除）

### 2. 主要能力

1. **接口列表 & 状态查看**
   - 枚举当前系统所有接口，显示：
     - 名称
     - 类型（Physical / Tun / Tap / WireGuard / Bridge / Veth / VLAN / Loopback）
     - 状态：UP/DOWN
     - IPv4 / IPv6 地址
   - 使用 `ip -o link show` + `ip -o addr show`（初版可以只用 `ip` 命令）

2. **虚拟接口管理**
   - 对 tun/tap、WireGuard、docker0、veth、bridge、VLAN 等接口：
     - 支持删除接口：
       - 删除前：
         - 清理接口 IP：
           - `ip addr del … dev IFACE`
         - 关闭接口：
           - `ip link set dev IFACE down`
         - 删除接口：
           - `ip link delete IFACE`
       - 支持 `dry-run` 模式（只打印、不执行）
     - 可以尝试查找关联进程（后端接口预留，可先实现简单版：按接口名 grep `/proc/*/cmdline`），但这一版可以只打印提示「可能需要手动检查相关进程」。

3. **物理网卡配置编辑**
   - 支持针对物理接口执行：
     - 切换 **静态 IP** / **DHCP**
     - 配置静态 IPv4（目标字段）：
       - IP地址：例如 `192.168.1.10`
       - 子网掩码：例如 `255.255.255.0`（内部转成 `/24` 前缀）
       - 网关：例如 `192.168.1.1`
       - DNS：多个地址，例如 `223.5.5.5, 114.114.114.114`
     - 物理接口 up/down：
       - `ip link set dev IFACE up/down`
   - **运行时修改**：
     - 使用 `ip addr/route` 在当前系统立即生效：
       - 先 `ip addr flush dev IFACE` 清旧地址
       - 再 `ip addr add IP/PREFIX dev IFACE`
       - 设置/替换默认路由：`ip route replace default via GATEWAY dev IFACE`
   - **持久修改（Netplan）**：
     - 解析 `/etc/netplan/*.yaml`
     - 找到对应接口（`network.ethernets.<iface>`）
     - 写入：
       - 静态 IP 模式：
         - `dhcp4: false`
         - `addresses: [ "192.168.1.10/24" ]`
         - `routes: [ { to: "default", via: "192.168.1.1" } ]`
         - `nameservers.addresses: [ "223.5.5.5", "114.114.114.114" ]`
       - DHCP 模式：
         - `dhcp4: true`
         - 删除 `addresses`、`routes`、`nameservers`
     - 保存前为原文件创建备份，例如：`xxx.yaml.bak-时间戳`
     - 保存后执行 `netplan apply` 或 `netplan try`：
       - 如果失败，打印错误并恢复备份。

4. **安全保护（重要！）**
   - 工具启动时检测当前用户 UID，非 root 则退出并提示。
   - 检测当前进程通过哪个接口上网：
     - 可以根据默认路由和本机 IP 粗略判断当前 SSH 所在接口。
   - 如果用户尝试修改 / down / 删除：
     - 正在承载 SSH 的物理接口；
     - 或唯一的默认路由接口；
     - 需要弹出 **非常明确的警告**，要求用户输入 `"YES"` 才继续。
   - 对 `lo` 和显然的物理接口删除操作，要限制或要求 `--force` 之类标志。

---

## 三、终端图形界面（TUI）设计

使用 `ratatui` + `crossterm` 实现一个简单但清晰的 TUI：

### 1. 布局

整个界面建议拆成 3 个区域：

1. **左侧：接口列表面板**
   - 按行列出所有接口，显示字段：
     - `[index] 名称  类型  UP/DOWN  IPv4(第一条)`
   - 支持通过 ↑/↓ 选择当前接口，高亮当前行。
   - 可按类型排序或分组（Physical 在前，虚拟在后）。

2. **右上：当前选中接口详情**
   - 显示：
     - 名称、类型、MAC、MTU、UP/DOWN
     - 所有 IPv4 / IPv6 地址
     - 对于物理接口：
       - 当前模式：静态 / DHCP
       - 当前网关、DNS（来自 Netplan 分析的视图）
     - 对于虚拟接口：
       - 简单标明：虚拟接口，不存在持久配置。

3. **右下：操作提示 / 表单区域**
   - 显示当前可用操作和快捷键：
     - 全局：
       - `q`：退出
       - `r`：刷新接口列表
     - 对物理接口：
       - `e`：编辑 IP/掩码/网关/DNS
       - `t`：切换 DHCP/静态
       - `u`：接口 up
       - `d`：接口 down
     - 对虚拟接口：
       - `x`：删除接口（清 IP + down + delete），弹确认。
   - 当用户进入“编辑模式”时，右下区域切换为表单：
     - 提示用户输入：
       - IP: `192.168.1.10`
       - Netmask: `255.255.255.0`
       - Gateway: `192.168.1.1`
       - DNS: `223.5.5.5,114.114.114.114`
     - 支持 Tab 在字段间切换，Enter 提交，Esc 取消。

### 2. TUI 事件与状态管理

请设计一个 `AppState` 结构体，包含：

```rust
enum Screen {
    Main,       // 正常浏览界面
    EditIface,  // 编辑当前接口配置的表单界面
    Confirm,    // 确认删除/危险操作界面
    Message,    // 弹出信息 / 错误窗口
}

struct AppState {
    interfaces: Vec<NetInterface>,
    selected_index: usize,

    screen: Screen,
    edit_form: Option<EditFormState>,
    confirm_state: Option<ConfirmState>,

    log_messages: Vec<String>, // 可在底部简单显示最近日志
    dry_run: bool,
}
````

* 主事件循环：

  * 使用 `crossterm` 监听键盘事件（`Event::Key`）
  * 每次事件更新 `AppState`，然后用 `ratatui` 重绘界面
  * 注意保持循环简洁，复杂逻辑放到单独函数里

---

## 四、后端模块划分建议

请按模块组织代码，结构清晰、易扩展：

1. `model.rs`

   * `NetInterface` 结构
   * `InterfaceKind` 枚举（Physical, Tun, Tap, WireGuard, Bridge, Veth, Vlan, Loopback）
   * `Ipv4Config`、`DnsConfig` 等模型

2. `backend/runtime.rs`

   * 与运行时 `ip` 命令交互：

     * `list_interfaces() -> Vec<NetInterface>`
     * `set_iface_up/down`
     * `set_iface_ipv4`
     * `delete_iface`
   * 使用 `Command::new("ip")` 调用，解析输出

3. `backend/netplan.rs`

   * 解析/修改 `/etc/netplan/*.yaml` 的模块：

     * `load_netplan_config()`
     * `get_iface_config(name)`
     * `update_iface_config(name, new_cfg)`
     * `save_and_apply(dry_run: bool)`
   * 用 `serde_yaml` 对 YAML 映射为 Rust 结构体（或使用 `serde_yaml::Value` 动态处理）

4. `ui.rs`

   * 使用 `ratatui` 渲染界面
   * 定义各种小组件：

     * 接口列表
     * 详情面板
     * 操作提示栏
     * 编辑表单
     * 确认对话框 / 消息弹框

5. `main.rs`

   * 使用 `clap` 解析命令行参数：

     * `--dry-run`
     * 将这些配置放入 `AppState`
   * 初始化 TUI，进入事件循环

---

## 五、其他要求

1. **必须给出完整可编译的 Rust 代码**：

   * `Cargo.toml`（列出依赖）
   * `src/main.rs` 和必要的模块文件
2. **代码中适量注释**，解释关键设计选择：

   * 为什么用某种布局
   * 接口类型的判定规则
   * Netplan 写入的注意事项
3. **使用说明**：

   * 如何构建：

     * `cargo build --release`
   * 如何运行：

     * `sudo ./target/release/netctl`（名称随你定，但尽量简单）
   * TUI 操作说明：

     * 按键列表、编辑流程、删除流程等

如果有相对复杂或耗时的操作（例如解析大 YAML、调用 `netplan apply`），可以在 UI 的底部显示一个简单的状态栏，如 `[INFO] 正在应用 Netplan 配置…`。

请按照以上需求，先给出：

1. 项目整体结构和设计说明
2. 完整的 `Cargo.toml`
3. 核心代码文件（可以是简化版但可以编译运行，至少包含：接口列表展示 + 选择 + 简单详情显示；Netplan 和删除接口可以先以 stub / TODO 实现，但要留出清晰接口方便后续扩展）

```
