# nicman 功能说明

## ✅ 已实现功能

### 1. 接口类型识别 (100%)

支持识别以下10种接口类型：

- 🔄 **Loopback** - 回环接口 (lo)
- 🔌 **Physical** - 物理网卡 (enp4s0, eno1等)
- 🐳 **Docker** - Docker网桥 (docker0, br-xxx等)
- 🔐 **WireGuard** - WireGuard VPN接口
- 🌉 **Bridge** - 网桥接口
- 🔗 **Veth** - Veth对
- 📡 **VLAN** - VLAN接口
- 🚇 **TUN** - TUN设备 (通过tun_flags检测)
- 🚰 **TAP** - TAP设备 (通过tun_flags检测)
- ❓ **Unknown** - 未知类型

**检测方法**：
- 通过`/sys/class/net/{iface}/tun_flags`文件检测TUN/TAP设备
- 通过`/sys/class/net/{iface}/device`检测物理网卡
- 通过`/sys/class/net/{iface}/bridge`目录检测网桥
- 通过接口名称前缀检测Docker、WireGuard等

### 2. 实时流量监控 (100%)

- ✅ 显示上下行速率（实时计算）
- ✅ 显示累计接收/发送字节数
- ✅ 显示累计接收/发送包数
- ✅ 自动单位转换（B/KB/MB/GB/TB）
- ✅ 每秒自动更新

**数据来源**：`/sys/class/net/{iface}/statistics/`

### 3. 创建者检测 (100%)

支持检测以下5种创建者类型：

- 🔧 **SystemdService** - systemd服务（wg-quick、openvpn等）
- 🐳 **DockerContainer** - Docker容器
- 🔨 **Process** - 进程（通过/proc检测）
- 📡 **NetworkManager** - NetworkManager连接
- 🧩 **Kernel** - 内核模块（bridge、8021q、wireguard）

**检测优先级**：Docker → systemd → Process → NetworkManager → Kernel

### 4. 接口操作 (100%)

#### 4.1 启用/禁用接口
- **快捷键**: `u` - 启用接口 (Up)
- **快捷键**: `D` - 禁用接口 (Down)
- **实现**: 使用`ip link set {iface} up/down`命令

#### 4.2 删除接口
- **快捷键**: `d` 或 `Delete` - 删除接口
- **确认对话框**: 显示接口信息、删除策略和安全警告
- **智能删除**: 根据创建者类型自动选择删除策略

**删除策略**：
- **InterfaceOnly** - 仅删除接口（虚拟接口）
- **StopService** - 停止systemd服务
- **StopAndDisableService** - 停止并禁用systemd服务
- **StopContainer** - 停止Docker容器
- **KillProcess** - 终止进程

**安全检查**：
- ⚠️ SSH连接接口警告
- ⚠️ 默认路由接口警告
- ⚠️ 活动连接警告

### 5. TUI界面 (100%)

#### 5.1 主界面
- **三栏布局**：
  - 左侧：接口列表（40%宽度）
  - 右上：接口详情
  - 右下：流量统计

#### 5.2 快捷键
**导航**：
- `↑` / `k` - 上移
- `↓` / `j` - 下移

**操作**：
- `r` - 刷新接口列表
- `u` - 启用接口
- `D` - 禁用接口
- `d` / `Delete` - 删除接口
- `q` - 退出程序
- `?` - 显示/隐藏帮助

**删除确认对话框**：
- `Y` - 确认删除
- `N` / `Esc` - 取消删除

#### 5.3 界面元素
- ✅ 接口图标（根据类型显示）
- ✅ 状态指示器（✅ Up / ❌ Down）
- ✅ 实时流量速率显示
- ✅ 选中项高亮显示
- ✅ 颜色编码（绿色=正常，红色=警告，黄色=提示）

### 6. 后端模块 (100%)

#### 6.1 runtime.rs - 运行时管理
- ✅ `list_interfaces()` - 列出所有接口
- ✅ `set_interface_up()` - 启用接口
- ✅ `set_interface_down()` - 禁用接口
- ✅ `delete_interface()` - 删除接口
- ✅ `set_ipv4_address()` - 设置IP地址
- ✅ `is_ssh_interface()` - 检查SSH连接

#### 6.2 traffic.rs - 流量监控
- ✅ `TrafficMonitor` - 流量监控器
- ✅ `update_all()` - 更新所有接口流量
- ✅ 实时速率计算

#### 6.3 owner_detection.rs - 创建者检测
- ✅ `OwnerDetector::detect()` - 检测接口创建者
- ✅ 多层次检测机制

#### 6.4 removal.rs - 智能删除
- ✅ `RemovalManager::determine_strategy()` - 确定删除策略
- ✅ `RemovalManager::remove_interface()` - 执行删除
- ✅ `RemovalManager::check_safety()` - 安全检查

#### 6.5 netplan.rs - Netplan配置管理
- ✅ 读取/写入Netplan配置
- ✅ 配置备份
- ✅ 设置静态IP/DHCP
- ⚠️ 注意：此功能已实现但未在UI中集成

---

## 🚧 未实现功能

### 1. IP配置编辑界面
- ❌ 交互式IP地址编辑表单
- ❌ DHCP/静态IP切换界面
- ❌ DNS配置编辑

### 2. 批量操作
- ❌ 批量启用/禁用接口
- ❌ 批量删除接口

### 3. 高级功能
- ❌ 搜索/过滤接口
- ❌ 配置导入/导出
- ❌ 历史记录查看
- ❌ 性能图表显示

---

## 📊 功能完成度

| 模块 | 完成度 | 说明 |
|------|--------|------|
| 接口类型识别 | 100% | 支持10种类型，包括TUN/TAP |
| 流量监控 | 100% | 实时速率计算和显示 |
| 创建者检测 | 100% | 5种检测方式 |
| 接口操作 | 100% | 启用/禁用/删除 |
| TUI界面 | 100% | 三栏布局，完整交互 |
| 智能删除 | 100% | 策略选择和安全检查 |
| Netplan管理 | 80% | 后端完成，UI未集成 |
| IP配置编辑 | 0% | 未实现 |
| 批量操作 | 0% | 未实现 |

**总体完成度**: **85%**

---

## 🎯 使用示例

### 启动程序
```bash
sudo nicman
```

### 查看接口列表
- 程序启动后自动显示所有接口
- 使用 `↑↓` 或 `jk` 键导航

### 启用/禁用接口
1. 选中要操作的接口
2. 按 `u` 启用或 `D` 禁用
3. 接口状态立即更新

### 删除接口
1. 选中要删除的接口
2. 按 `d` 或 `Delete`
3. 查看删除确认对话框
4. 按 `Y` 确认或 `N` 取消

### 查看帮助
- 按 `?` 显示帮助界面
- 再次按 `?` 或 `Esc` 返回主界面

---

**更新时间**: 2025-11-14 20:30:00  
**版本**: v0.1.0

