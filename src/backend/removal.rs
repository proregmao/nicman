// 智能删除模块 - 智能删除虚拟接口并防止自动重启
use crate::backend::runtime;
use crate::model::{InterfaceOwner, NetInterface, RemovalStrategy};
use crate::utils::command::{command_success, execute_command_stdout};
use anyhow::{Context, Result};

/// 接口删除管理器
pub struct RemovalManager;

impl RemovalManager {
    /// 确定删除策略
    pub fn determine_strategy(iface: &NetInterface) -> RemovalStrategy {
        match &iface.owner {
            Some(InterfaceOwner::SystemdService { .. }) => {
                RemovalStrategy::StopAndDisableService
            }
            Some(InterfaceOwner::DockerContainer { .. }) => {
                RemovalStrategy::StopContainer
            }
            Some(InterfaceOwner::Process { .. }) => {
                RemovalStrategy::KillProcess
            }
            Some(InterfaceOwner::NetworkManager { .. }) => {
                RemovalStrategy::StopService
            }
            _ => RemovalStrategy::InterfaceOnly,
        }
    }

    /// 执行删除操作
    pub fn remove_interface(iface: &NetInterface, strategy: &RemovalStrategy) -> Result<()> {
        match strategy {
            RemovalStrategy::InterfaceOnly => {
                Self::remove_interface_only(&iface.name)
            }
            RemovalStrategy::StopService => {
                Self::stop_service(iface)?;
                Self::remove_interface_only(&iface.name)
            }
            RemovalStrategy::StopAndDisableService => {
                Self::stop_and_disable_service(iface)?;
                Self::remove_interface_only(&iface.name)
            }
            RemovalStrategy::StopContainer => {
                Self::stop_container(iface)?;
                Self::remove_interface_only(&iface.name)
            }
            RemovalStrategy::KillProcess => {
                Self::kill_process(iface)?;
                Self::remove_interface_only(&iface.name)
            }
        }
    }

    /// 仅删除接口（不处理创建者）
    fn remove_interface_only(iface_name: &str) -> Result<()> {
        runtime::delete_interface(iface_name)
            .with_context(|| format!("删除接口 {} 失败", iface_name))
    }

    /// 停止systemd服务
    fn stop_service(iface: &NetInterface) -> Result<()> {
        if let Some(InterfaceOwner::SystemdService { name, .. }) = &iface.owner {
            execute_command_stdout("systemctl", &["stop", name])
                .with_context(|| format!("停止服务 {} 失败", name))?;
            println!("✅ 已停止服务: {}", name);
        }
        Ok(())
    }

    /// 停止并禁用systemd服务
    fn stop_and_disable_service(iface: &NetInterface) -> Result<()> {
        if let Some(InterfaceOwner::SystemdService { name, .. }) = &iface.owner {
            // 停止服务
            execute_command_stdout("systemctl", &["stop", name])
                .with_context(|| format!("停止服务 {} 失败", name))?;
            println!("✅ 已停止服务: {}", name);

            // 禁用服务（防止开机自启）
            execute_command_stdout("systemctl", &["disable", name])
                .with_context(|| format!("禁用服务 {} 失败", name))?;
            println!("✅ 已禁用服务: {}", name);
        }
        Ok(())
    }

    /// 停止Docker容器
    fn stop_container(iface: &NetInterface) -> Result<()> {
        if let Some(InterfaceOwner::DockerContainer { id, name, .. }) = &iface.owner {
            if id == "system" {
                // Docker网桥不能停止
                return Ok(());
            }

            if command_success("docker", &["stop", id]) {
                println!("✅ 已停止容器: {} ({})", name, id);
            } else {
                println!("⚠️ 停止容器失败: {} ({})", name, id);
            }
        }
        Ok(())
    }

    /// 终止进程
    fn kill_process(iface: &NetInterface) -> Result<()> {
        if let Some(InterfaceOwner::Process { pid, name, .. }) = &iface.owner {
            // 先尝试SIGTERM（优雅终止）
            if command_success("kill", &[&pid.to_string()]) {
                println!("✅ 已发送SIGTERM信号到进程: {} (PID: {})", name, pid);

                // 等待1秒
                std::thread::sleep(std::time::Duration::from_secs(1));

                // 检查进程是否还存在
                if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                    // 进程仍存在，使用SIGKILL强制终止
                    if command_success("kill", &["-9", &pid.to_string()]) {
                        println!("✅ 已发送SIGKILL信号到进程: {} (PID: {})", name, pid);
                    }
                }
            } else {
                println!("⚠️ 终止进程失败: {} (PID: {})", name, pid);
            }
        }
        Ok(())
    }

    /// 检查删除前的安全性
    pub fn check_safety(iface: &NetInterface) -> Vec<String> {
        let mut warnings = Vec::new();

        // 检查是否是SSH连接接口
        if runtime::is_ssh_interface(&iface.name) {
            warnings.push(format!("⚠️ 警告: {} 是SSH连接使用的接口，删除后可能导致远程连接断开！", iface.name));
        }

        // 检查是否是唯一的默认路由接口
        if let Ok(Some(default_iface)) = runtime::get_default_route_interface() {
            if default_iface == iface.name {
                warnings.push(format!("⚠️ 警告: {} 是默认路由接口，删除后可能无法访问外网！", iface.name));
            }
        }

        // 检查是否有活跃的连接
        if !iface.ipv4_addresses.is_empty() || !iface.ipv6_addresses.is_empty() {
            warnings.push(format!("⚠️ 提示: {} 配置了IP地址，可能有活跃的网络连接", iface.name));
        }

        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{InterfaceKind, InterfaceState};

    #[test]
    fn test_determine_strategy() {
        let mut iface = NetInterface::new("test0".to_string(), InterfaceKind::Tun);

        // 测试systemd服务策略
        iface.owner = Some(InterfaceOwner::SystemdService {
            name: "test.service".to_string(),
            status: crate::model::ServiceStatus::Active,
            start_time: None,
        });
        assert!(matches!(
            RemovalManager::determine_strategy(&iface),
            RemovalStrategy::StopAndDisableService
        ));

        // 测试Docker容器策略
        iface.owner = Some(InterfaceOwner::DockerContainer {
            id: "abc123".to_string(),
            name: "test-container".to_string(),
            image: "test:latest".to_string(),
        });
        assert!(matches!(
            RemovalManager::determine_strategy(&iface),
            RemovalStrategy::StopContainer
        ));
    }
}

