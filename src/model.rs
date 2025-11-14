// 数据模型定义
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 网络接口类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceKind {
    Physical,      // 物理网卡
    Loopback,      // 回环接口
    Tun,           // TUN设备
    Tap,           // TAP设备
    WireGuard,     // WireGuard VPN
    Bridge,        // 网桥
    Veth,          // 虚拟以太网对
    Vlan,          // VLAN接口
    Docker,        // Docker网桥
    Unknown,       // 未知类型
}

impl InterfaceKind {
    /// 判断是否为虚拟接口
    #[allow(dead_code)]
    pub fn is_virtual(&self) -> bool {
        !matches!(self, InterfaceKind::Physical | InterfaceKind::Loopback)
    }

    /// 获取类型的显示名称
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            InterfaceKind::Physical => "物理网卡",
            InterfaceKind::Loopback => "回环接口",
            InterfaceKind::Tun => "TUN设备",
            InterfaceKind::Tap => "TAP设备",
            InterfaceKind::WireGuard => "WireGuard",
            InterfaceKind::Bridge => "网桥",
            InterfaceKind::Veth => "虚拟以太网",
            InterfaceKind::Vlan => "VLAN",
            InterfaceKind::Docker => "Docker网桥",
            InterfaceKind::Unknown => "未知",
        }
    }

    /// 获取类型的图标
    #[allow(dead_code)]
    pub fn icon(&self) -> &str {
        match self {
            InterfaceKind::Physical => "🔌",
            InterfaceKind::Loopback => "🔄",
            InterfaceKind::Tun | InterfaceKind::Tap => "🔐",
            InterfaceKind::WireGuard => "🔒",
            InterfaceKind::Bridge => "🌉",
            InterfaceKind::Veth => "🔗",
            InterfaceKind::Vlan => "🏷️",
            InterfaceKind::Docker => "🐳",
            InterfaceKind::Unknown => "❓",
        }
    }
}

/// 接口状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceState {
    Up,
    Down,
    Unknown,
}

impl InterfaceState {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            InterfaceState::Up => "UP",
            InterfaceState::Down => "DOWN",
            InterfaceState::Unknown => "UNKNOWN",
        }
    }
}

/// 流量统计数据
#[derive(Debug, Clone)]
pub struct TrafficStats {
    pub rx_bytes: u64,       // 接收字节数
    pub tx_bytes: u64,       // 发送字节数
    pub rx_packets: u64,     // 接收包数
    pub tx_packets: u64,     // 发送包数
    #[allow(dead_code)]
    pub rx_errors: u64,      // 接收错误
    #[allow(dead_code)]
    pub tx_errors: u64,      // 发送错误
    #[allow(dead_code)]
    pub rx_dropped: u64,     // 接收丢包
    #[allow(dead_code)]
    pub tx_dropped: u64,     // 发送丢包
    pub rx_speed: f64,       // 接收速率 (bytes/sec)
    pub tx_speed: f64,       // 发送速率 (bytes/sec)
    pub last_update: Instant, // 最后更新时间
}

impl Default for TrafficStats {
    fn default() -> Self {
        Self {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_errors: 0,
            tx_errors: 0,
            rx_dropped: 0,
            tx_dropped: 0,
            rx_speed: 0.0,
            tx_speed: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// 服务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Unknown,
}

/// 接口创建者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceOwner {
    SystemdService {
        name: String,
        status: ServiceStatus,
        start_time: Option<String>,
    },
    DockerContainer {
        id: String,
        name: String,
        image: String,
    },
    Process {
        pid: u32,
        name: String,
        cmdline: String,
    },
    NetworkManager {
        connection: String,
        uuid: String,
    },
    Kernel {
        module: String,
    },
    Unknown,
}

impl InterfaceOwner {
    /// 获取创建者的显示名称
    pub fn display_name(&self) -> String {
        match self {
            InterfaceOwner::SystemdService { name, .. } => format!("systemd: {}", name),
            InterfaceOwner::DockerContainer { name, .. } => format!("Docker: {}", name),
            InterfaceOwner::Process { name, pid, .. } => format!("进程: {} (PID: {})", name, pid),
            InterfaceOwner::NetworkManager { connection, .. } => format!("NetworkManager: {}", connection),
            InterfaceOwner::Kernel { module } => format!("内核模块: {}", module),
            InterfaceOwner::Unknown => "未知".to_string(),
        }
    }

    /// 获取创建者的图标
    #[allow(dead_code)]
    pub fn icon(&self) -> &str {
        match self {
            InterfaceOwner::SystemdService { .. } => "📦",
            InterfaceOwner::DockerContainer { .. } => "🐳",
            InterfaceOwner::Process { .. } => "⚙️",
            InterfaceOwner::NetworkManager { .. } => "🔧",
            InterfaceOwner::Kernel { .. } => "🐧",
            InterfaceOwner::Unknown => "❓",
        }
    }
}

/// IP配置模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpConfigMode {
    Static,
    Dhcp,
    None,
}

/// IPv4配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv4Config {
    pub address: String,      // IP地址
    pub netmask: String,      // 子网掩码
    pub prefix: u8,           // 前缀长度 (如24)
    pub gateway: Option<String>, // 网关
}

/// DNS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub nameservers: Vec<String>, // DNS服务器列表
}

/// 网络接口完整信息
#[derive(Debug, Clone)]
pub struct NetInterface {
    pub name: String,                    // 接口名称
    pub kind: InterfaceKind,             // 接口类型
    pub state: InterfaceState,           // 接口状态
    pub mac_address: Option<String>,     // MAC地址
    pub mtu: u32,                        // MTU
    pub ipv4_addresses: Vec<String>,     // IPv4地址列表
    pub ipv6_addresses: Vec<String>,     // IPv6地址列表
    pub traffic_stats: TrafficStats,     // 流量统计
    pub owner: Option<InterfaceOwner>,   // 创建者信息
    #[allow(dead_code)]
    pub config_mode: IpConfigMode,       // 配置模式
    #[allow(dead_code)]
    pub ipv4_config: Option<Ipv4Config>, // IPv4配置
    #[allow(dead_code)]
    pub dns_config: Option<DnsConfig>,   // DNS配置
}

impl NetInterface {
    /// 创建新的接口实例
    pub fn new(name: String, kind: InterfaceKind) -> Self {
        Self {
            name,
            kind,
            state: InterfaceState::Unknown,
            mac_address: None,
            mtu: 1500,
            ipv4_addresses: Vec::new(),
            ipv6_addresses: Vec::new(),
            traffic_stats: TrafficStats::default(),
            owner: None,
            config_mode: IpConfigMode::None,
            ipv4_config: None,
            dns_config: None,
        }
    }

    /// 获取第一个IPv4地址（用于列表显示）
    #[allow(dead_code)]
    pub fn primary_ipv4(&self) -> Option<&String> {
        self.ipv4_addresses.first()
    }

    /// 判断是否可以删除
    #[allow(dead_code)]
    pub fn is_deletable(&self) -> bool {
        self.kind.is_virtual() && self.kind != InterfaceKind::Loopback
    }

    /// 判断是否可以编辑IP配置
    #[allow(dead_code)]
    pub fn is_configurable(&self) -> bool {
        self.kind == InterfaceKind::Physical
    }
}

/// 删除策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemovalStrategy {
    /// 仅删除接口（临时，可能被重新创建）
    InterfaceOnly,
    /// 停止服务并删除接口（推荐）
    StopService,
    /// 停止、禁用服务并删除接口（永久）
    StopAndDisableService,
    /// 停止容器并删除接口
    StopContainer,
    /// 终止进程并删除接口
    KillProcess,
}

impl RemovalStrategy {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            RemovalStrategy::InterfaceOnly => "仅删除接口（临时）",
            RemovalStrategy::StopService => "停止服务并删除（推荐）",
            RemovalStrategy::StopAndDisableService => "停止并禁用服务（永久）",
            RemovalStrategy::StopContainer => "停止容器",
            RemovalStrategy::KillProcess => "终止进程",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        match self {
            RemovalStrategy::InterfaceOnly => "仅删除接口，服务仍在运行，接口可能立即重建",
            RemovalStrategy::StopService => "停止服务并删除接口，服务仍会开机自启",
            RemovalStrategy::StopAndDisableService => "停止服务、禁用开机自启并删除接口",
            RemovalStrategy::StopContainer => "停止Docker容器，接口会自动删除",
            RemovalStrategy::KillProcess => "终止持有接口的进程",
        }
    }
}

