// 运行时接口管理模块 - 使用ip命令管理网络接口
use crate::model::{InterfaceKind, InterfaceState, NetInterface};
use crate::utils::command::execute_command_stdout;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;

/// 列出所有网络接口
pub fn list_interfaces() -> Result<Vec<NetInterface>> {
    let mut interfaces = Vec::new();

    // 使用 ip -o link show 获取接口列表
    let output = execute_command_stdout("ip", &["-o", "link", "show"])?;

    for line in output.lines() {
        if let Some(iface) = parse_interface_from_link(line)? {
            interfaces.push(iface);
        }
    }

    // 为每个接口添加IP地址信息
    for iface in &mut interfaces {
        add_ip_addresses(iface)?;
    }

    // 检测接口创建者
    use crate::backend::owner_detection::OwnerDetector;
    for iface in &mut interfaces {
        iface.owner = OwnerDetector::detect(iface);
    }

    Ok(interfaces)
}

/// 从 ip link show 输出解析接口信息
fn parse_interface_from_link(line: &str) -> Result<Option<NetInterface>> {
    // 示例输出: 2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc ...
    let re = Regex::new(r"^\d+:\s+([^:@]+)[@:]?\s*<([^>]*)>\s+.*mtu\s+(\d+)")?;

    if let Some(caps) = re.captures(line) {
        let name = caps.get(1).unwrap().as_str().trim().to_string();
        let flags = caps.get(2).unwrap().as_str();
        let mtu: u32 = caps.get(3).unwrap().as_str().parse()?;

        // 判断接口类型
        let kind = detect_interface_kind(&name)?;

        // 判断接口状态
        let state = if flags.contains("UP") {
            InterfaceState::Up
        } else {
            InterfaceState::Down
        };

        // 获取MAC地址
        let mac_address = extract_mac_address(line);

        let mut iface = NetInterface::new(name, kind);
        iface.state = state;
        iface.mtu = mtu;
        iface.mac_address = mac_address;

        Ok(Some(iface))
    } else {
        Ok(None)
    }
}

/// 检测接口类型
fn detect_interface_kind(name: &str) -> Result<InterfaceKind> {
    // 首先检查 /sys/class/net/{name}/type
    let type_path = format!("/sys/class/net/{}/type", name);
    let uevent_path = format!("/sys/class/net/{}/uevent", name);

    // 回环接口
    if name == "lo" {
        return Ok(InterfaceKind::Loopback);
    }

    // 检查是否是Docker网桥
    if name == "docker0" || name.starts_with("br-") {
        return Ok(InterfaceKind::Docker);
    }

    // 检查是否是WireGuard
    if name.starts_with("wg") {
        if let Ok(uevent) = fs::read_to_string(&uevent_path) {
            if uevent.contains("wireguard") {
                return Ok(InterfaceKind::WireGuard);
            }
        }
    }

    // 检查是否是veth
    if name.starts_with("veth") {
        return Ok(InterfaceKind::Veth);
    }

    // 检查是否是VLAN (格式: eth0.10)
    if name.contains('.') {
        return Ok(InterfaceKind::Vlan);
    }

    // 检查是否是网桥
    let bridge_path = format!("/sys/class/net/{}/bridge", name);
    if fs::metadata(&bridge_path).is_ok() {
        return Ok(InterfaceKind::Bridge);
    }

    // 检查是否是tun/tap（通过tun_flags文件判断）
    let tun_flags_path = format!("/sys/class/net/{}/tun_flags", name);
    if let Ok(flags_str) = fs::read_to_string(&tun_flags_path) {
        let flags = flags_str.trim();
        // tun_flags存在表示是tun/tap设备
        // 0x1 = TUN, 0x2 = TAP
        if flags == "0x1" || flags == "1" {
            return Ok(InterfaceKind::Tun);
        } else if flags == "0x2" || flags == "2" {
            return Ok(InterfaceKind::Tap);
        }
    }

    // 兼容：通过名称前缀判断
    if name.starts_with("tun") {
        return Ok(InterfaceKind::Tun);
    }
    if name.starts_with("tap") {
        return Ok(InterfaceKind::Tap);
    }

    // 检查type文件判断是否是物理网卡
    if let Ok(type_str) = fs::read_to_string(&type_path) {
        let type_num: u32 = type_str.trim().parse().unwrap_or(0);
        // type 1 = 以太网
        if type_num == 1 {
            // 进一步检查是否有物理设备
            let device_path = format!("/sys/class/net/{}/device", name);
            if fs::metadata(&device_path).is_ok() {
                return Ok(InterfaceKind::Physical);
            }
        }
    }

    // 默认返回Unknown
    Ok(InterfaceKind::Unknown)
}

/// 从输出中提取MAC地址
fn extract_mac_address(line: &str) -> Option<String> {
    let re = Regex::new(r"link/ether\s+([0-9a-f:]{17})").ok()?;
    re.captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// 为接口添加IP地址信息
fn add_ip_addresses(iface: &mut NetInterface) -> Result<()> {
    let output = execute_command_stdout("ip", &["-o", "addr", "show", "dev", &iface.name])?;

    for line in output.lines() {
        // 示例: 2: eth0    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0
        if line.contains("inet ") {
            if let Some(addr) = extract_ipv4_address(line) {
                iface.ipv4_addresses.push(addr.clone());

                // 解析IP地址和前缀，填充ipv4_config
                if let Some((ip, prefix_str)) = addr.split_once('/') {
                    if let Ok(prefix) = prefix_str.parse::<u8>() {
                        use crate::model::Ipv4Config;
                        iface.ipv4_config = Some(Ipv4Config {
                            address: ip.to_string(),
                            netmask: prefix_to_netmask(prefix),
                            prefix,
                            gateway: get_default_gateway(&iface.name).ok(),
                        });
                    }
                }
            }
        } else if line.contains("inet6 ") {
            if let Some(addr) = extract_ipv6_address(line) {
                iface.ipv6_addresses.push(addr);
            }
        }
    }

    // 读取DNS配置
    if let Ok(dns_servers) = get_dns_servers() {
        if !dns_servers.is_empty() {
            use crate::model::DnsConfig;
            iface.dns_config = Some(DnsConfig {
                nameservers: dns_servers,
            });
        }
    }

    Ok(())
}

/// 提取IPv4地址
fn extract_ipv4_address(line: &str) -> Option<String> {
    let re = Regex::new(r"inet\s+([0-9.]+/\d+)").ok()?;
    re.captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// 提取IPv6地址
fn extract_ipv6_address(line: &str) -> Option<String> {
    let re = Regex::new(r"inet6\s+([0-9a-f:]+/\d+)").ok()?;
    re.captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// 将前缀长度转换为子网掩码
fn prefix_to_netmask(prefix: u8) -> String {
    if prefix > 32 {
        return "255.255.255.255".to_string();
    }

    let mask: u32 = if prefix == 0 {
        0
    } else {
        !0u32 << (32 - prefix)
    };

    format!(
        "{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF
    )
}

/// 获取默认网关
fn get_default_gateway(iface_name: &str) -> Result<String> {
    let output = execute_command_stdout("ip", &["route", "show", "default", "dev", iface_name])?;

    // 示例输出: default via 192.168.1.1 dev enp4s0 proto static
    let re = Regex::new(r"default via ([0-9.]+)")?;
    if let Some(caps) = re.captures(&output) {
        if let Some(gateway) = caps.get(1) {
            return Ok(gateway.as_str().to_string());
        }
    }

    Err(anyhow::anyhow!("未找到默认网关"))
}

/// 获取DNS服务器列表
fn get_dns_servers() -> Result<Vec<String>> {
    let mut dns_servers = Vec::new();

    // 尝试从 /etc/resolv.conf 读取
    if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
        let re = Regex::new(r"nameserver\s+([0-9.]+)")?;
        for line in content.lines() {
            if let Some(caps) = re.captures(line) {
                if let Some(dns) = caps.get(1) {
                    dns_servers.push(dns.as_str().to_string());
                }
            }
        }
    }

    Ok(dns_servers)
}

/// 设置接口状态为UP
pub fn set_interface_up(iface_name: &str) -> Result<()> {
    execute_command_stdout("ip", &["link", "set", "dev", iface_name, "up"])
        .with_context(|| format!("启用接口 {} 失败", iface_name))?;
    Ok(())
}

/// 设置接口状态为DOWN
pub fn set_interface_down(iface_name: &str) -> Result<()> {
    execute_command_stdout("ip", &["link", "set", "dev", iface_name, "down"])
        .with_context(|| format!("禁用接口 {} 失败", iface_name))?;
    Ok(())
}

/// 删除接口
pub fn delete_interface(iface_name: &str) -> Result<()> {
    execute_command_stdout("ip", &["link", "delete", iface_name])
        .with_context(|| format!("删除接口 {} 失败", iface_name))?;
    Ok(())
}

/// 为接口设置IPv4地址
pub fn set_ipv4_address(iface_name: &str, address: &str, prefix: u8) -> Result<()> {
    let addr_with_prefix = format!("{}/{}", address, prefix);
    execute_command_stdout("ip", &["addr", "add", &addr_with_prefix, "dev", iface_name])
        .with_context(|| format!("设置接口 {} 的IP地址失败", iface_name))?;
    Ok(())
}

/// 清除接口的所有IPv4地址
pub fn flush_ipv4_addresses(iface_name: &str) -> Result<()> {
    execute_command_stdout("ip", &["addr", "flush", "dev", iface_name])
        .with_context(|| format!("清除接口 {} 的IP地址失败", iface_name))?;
    Ok(())
}

/// 设置默认网关
pub fn set_default_gateway(gateway: &str, iface_name: &str) -> Result<()> {
    execute_command_stdout("ip", &["route", "replace", "default", "via", gateway, "dev", iface_name])
        .with_context(|| format!("设置默认网关失败"))?;
    Ok(())
}

/// 获取默认路由接口
pub fn get_default_route_interface() -> Result<Option<String>> {
    let output = execute_command_stdout("ip", &["route", "show", "default"])?;

    // 示例输出: default via 192.168.1.1 dev eth0 proto dhcp metric 100
    let re = Regex::new(r"dev\s+(\S+)")?;
    if let Some(caps) = re.captures(&output) {
        Ok(Some(caps.get(1).unwrap().as_str().to_string()))
    } else {
        Ok(None)
    }
}

/// 检查是否是SSH连接使用的接口
pub fn is_ssh_interface(iface_name: &str) -> bool {
    // 检查SSH_CONNECTION环境变量
    if let Ok(ssh_conn) = std::env::var("SSH_CONNECTION") {
        let parts: Vec<&str> = ssh_conn.split_whitespace().collect();
        if parts.len() >= 3 {
            let local_ip = parts[2];
            // 检查这个IP是否属于该接口
            if let Ok(output) = execute_command_stdout("ip", &["addr", "show", "dev", iface_name]) {
                return output.contains(local_ip);
            }
        }
    }

    // 检查是否是默认路由接口
    if let Ok(Some(default_iface)) = get_default_route_interface() {
        return default_iface == iface_name;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_interface_kind() {
        assert_eq!(detect_interface_kind("lo").unwrap(), InterfaceKind::Loopback);
        assert_eq!(detect_interface_kind("docker0").unwrap(), InterfaceKind::Docker);
        assert_eq!(detect_interface_kind("veth1234").unwrap(), InterfaceKind::Veth);
        assert_eq!(detect_interface_kind("eth0.10").unwrap(), InterfaceKind::Vlan);
    }

    #[test]
    fn test_extract_ipv4_address() {
        let line = "2: eth0    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0";
        assert_eq!(extract_ipv4_address(line), Some("192.168.1.100/24".to_string()));
    }
}

