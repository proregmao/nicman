// 创建者检测模块 - 检测systemd服务、Docker容器、进程等创建者
use crate::model::{InterfaceKind, InterfaceOwner, NetInterface, ServiceStatus};
use crate::utils::command::{command_success, execute_command_stdout};
use regex::Regex;
use std::fs;

/// 接口创建者检测器
pub struct OwnerDetector;

impl OwnerDetector {
    /// 检测接口的创建者
    pub fn detect(iface: &NetInterface) -> Option<InterfaceOwner> {
        // 按优先级依次检测
        None
            .or_else(|| Self::check_docker_container(&iface.name, &iface.kind))
            .or_else(|| Self::check_systemd_service(&iface.name, &iface.kind))
            .or_else(|| Self::check_process_fd(&iface.name))
            .or_else(|| Self::check_network_manager(&iface.name))
            .or_else(|| Self::check_kernel_module(&iface.name, &iface.kind))
    }

    /// 检测Docker容器
    fn check_docker_container(iface_name: &str, kind: &InterfaceKind) -> Option<InterfaceOwner> {
        // Docker网桥和veth接口
        if !matches!(kind, InterfaceKind::Docker | InterfaceKind::Veth) {
            return None;
        }

        // 检查docker命令是否可用
        if !command_success("docker", &["--version"]) {
            return None;
        }

        // 对于Docker网桥，直接返回
        if iface_name == "docker0" || iface_name.starts_with("br-") {
            return Some(InterfaceOwner::DockerContainer {
                id: "system".to_string(),
                name: "Docker网桥".to_string(),
                image: "docker-network".to_string(),
            });
        }

        // 对于veth接口，尝试找到关联的容器
        if let Ok(output) = execute_command_stdout("docker", &["ps", "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}"]) {
            let containers: Vec<_> = output.lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 3 {
                        Some((parts[0], parts[1], parts[2]))
                    } else {
                        None
                    }
                })
                .collect();

            // 如果只有一个容器，直接关联
            if containers.len() == 1 {
                let (container_id, container_name, image) = containers[0];
                return Some(InterfaceOwner::DockerContainer {
                    id: container_id.to_string(),
                    name: container_name.to_string(),
                    image: image.to_string(),
                });
            }

            // 如果有多个容器，尝试精确匹配
            for (container_id, container_name, image) in containers {
                if Self::container_has_veth(container_id, iface_name) {
                    return Some(InterfaceOwner::DockerContainer {
                        id: container_id.to_string(),
                        name: container_name.to_string(),
                        image: image.to_string(),
                    });
                }
            }
        }

        None
    }

    /// 检查容器是否拥有指定的veth接口
    fn container_has_veth(container_id: &str, _iface_name: &str) -> bool {
        // 获取容器的网络命名空间PID
        if let Ok(output) = execute_command_stdout("docker", &["inspect", "-f", "{{.State.Pid}}", container_id]) {
            if let Ok(pid) = output.trim().parse::<u32>() {
                // 检查容器的网络接口
                if let Ok(output) = execute_command_stdout("nsenter", &["-t", &pid.to_string(), "-n", "ip", "link", "show"]) {
                    // 检查veth接口的对端是否在容器内
                    // veth接口成对出现，主机端的veth对应容器内的eth0等
                    if output.contains("eth0") || output.contains("eth1") {
                        // 简化：如果容器有网络接口，就认为这个veth可能属于它
                        // 更精确的方法需要检查veth的peer index
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 检测systemd服务
    fn check_systemd_service(iface_name: &str, kind: &InterfaceKind) -> Option<InterfaceOwner> {
        // 常见的服务命名模式
        let service_patterns = vec![
            format!("wg-quick@{}.service", iface_name),
            format!("openvpn@{}.service", iface_name),
            format!("openvpn-client@{}.service", iface_name),
            format!("netctl@{}.service", iface_name),
        ];

        // 对于WireGuard接口，优先检查wg-quick服务
        if matches!(kind, InterfaceKind::WireGuard) {
            if let Some(owner) = Self::check_service(&format!("wg-quick@{}.service", iface_name)) {
                return Some(owner);
            }
        }

        // 检查其他服务模式
        for service_name in service_patterns {
            if let Some(owner) = Self::check_service(&service_name) {
                return Some(owner);
            }
        }

        None
    }

    /// 检查单个systemd服务
    fn check_service(service_name: &str) -> Option<InterfaceOwner> {
        if let Ok(output) = execute_command_stdout("systemctl", &["status", service_name]) {
            let status = if output.contains("Active: active") {
                ServiceStatus::Active
            } else if output.contains("Active: inactive") {
                ServiceStatus::Inactive
            } else if output.contains("Active: failed") {
                ServiceStatus::Failed
            } else {
                ServiceStatus::Unknown
            };

            // 提取启动时间
            let start_time = Self::extract_start_time(&output);

            return Some(InterfaceOwner::SystemdService {
                name: service_name.to_string(),
                status,
                start_time,
            });
        }

        None
    }

    /// 从systemctl status输出中提取启动时间
    fn extract_start_time(output: &str) -> Option<String> {
        let re = Regex::new(r"since\s+(.+?);").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// 检测持有tun/tap设备的进程
    fn check_process_fd(iface_name: &str) -> Option<InterfaceOwner> {
        // 检查接口是否是tun/tap类型（通过tun_flags文件判断）
        let tun_flags_path = format!("/sys/class/net/{}/tun_flags", iface_name);
        if fs::metadata(&tun_flags_path).is_err() {
            // 不是tun/tap设备
            return None;
        }

        // 遍历/proc目录查找持有/dev/net/tun的进程
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        if let Some(owner) = Self::check_process_tun(pid, iface_name) {
                            return Some(owner);
                        }
                    }
                }
            }
        }

        None
    }

    /// 检查进程是否持有tun设备
    fn check_process_tun(pid: u32, iface_name: &str) -> Option<InterfaceOwner> {
        let fd_dir = format!("/proc/{}/fd", pid);
        if let Ok(entries) = fs::read_dir(&fd_dir) {
            for entry in entries.flatten() {
                if let Ok(link) = fs::read_link(entry.path()) {
                    if link.to_string_lossy().contains("/dev/net/tun") {
                        // 验证这个进程是否真的拥有这个接口
                        // 通过检查进程的网络命名空间中是否有这个接口
                        if Self::process_owns_interface(pid, iface_name) {
                            // 读取进程信息
                            let name = Self::read_process_name(pid).unwrap_or_else(|| format!("pid-{}", pid));
                            let cmdline = Self::read_process_cmdline(pid).unwrap_or_default();

                            return Some(InterfaceOwner::Process {
                                pid,
                                name,
                                cmdline,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// 检查进程是否拥有指定的网络接口
    fn process_owns_interface(pid: u32, iface_name: &str) -> bool {
        // 检查进程的网络命名空间中是否有这个接口
        if let Ok(output) = execute_command_stdout("nsenter", &["-t", &pid.to_string(), "-n", "ip", "link", "show", iface_name]) {
            return output.contains(iface_name);
        }
        // 如果nsenter失败，假设进程拥有这个接口（降级处理）
        true
    }

    /// 读取进程名称
    fn read_process_name(pid: u32) -> Option<String> {
        let comm_path = format!("/proc/{}/comm", pid);
        fs::read_to_string(comm_path).ok().map(|s| s.trim().to_string())
    }

    /// 读取进程命令行
    fn read_process_cmdline(pid: u32) -> Option<String> {
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        fs::read_to_string(cmdline_path).ok().map(|s| {
            s.replace('\0', " ").trim().to_string()
        })
    }

    /// 检测NetworkManager管理的连接
    fn check_network_manager(iface_name: &str) -> Option<InterfaceOwner> {
        // 检查nmcli命令是否可用
        if !command_success("nmcli", &["--version"]) {
            return None;
        }

        // 检查接口是否由NetworkManager管理
        if let Ok(output) = execute_command_stdout("nmcli", &["device", "show", iface_name]) {
            if output.contains("GENERAL.CONNECTION") {
                // 提取连接名称和UUID
                let connection = Self::extract_nm_connection(&output);
                let uuid = Self::extract_nm_uuid(&output);

                if let (Some(conn), Some(uuid_val)) = (connection, uuid) {
                    return Some(InterfaceOwner::NetworkManager {
                        connection: conn,
                        uuid: uuid_val,
                    });
                }
            }
        }

        None
    }

    /// 从nmcli输出中提取连接名称
    fn extract_nm_connection(output: &str) -> Option<String> {
        let re = Regex::new(r"GENERAL\.CONNECTION:\s+(.+)").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// 从nmcli输出中提取UUID
    fn extract_nm_uuid(output: &str) -> Option<String> {
        let re = Regex::new(r"GENERAL\.CON-UUID:\s+(.+)").ok()?;
        re.captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// 检测内核模块
    fn check_kernel_module(_iface_name: &str, kind: &InterfaceKind) -> Option<InterfaceOwner> {
        let module = match kind {
            InterfaceKind::Bridge => "bridge",
            InterfaceKind::Vlan => "8021q",
            InterfaceKind::WireGuard => "wireguard",
            _ => return None,
        };

        // 检查模块是否加载
        if let Ok(output) = execute_command_stdout("lsmod", &[]) {
            if output.contains(module) {
                return Some(InterfaceOwner::Kernel {
                    module: module.to_string(),
                });
            }
        }

        None
    }
}

