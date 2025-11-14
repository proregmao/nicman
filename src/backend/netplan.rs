// Netplan配置管理模块 - 管理持久化网络配置
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Netplan配置管理器
pub struct NetplanManager {
    config_dir: PathBuf,
}

impl NetplanManager {
    /// 创建新的Netplan管理器
    pub fn new() -> Self {
        Self {
            config_dir: PathBuf::from("/etc/netplan"),
        }
    }

    /// 列出所有Netplan配置文件
    pub fn list_config_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !self.config_dir.exists() {
            return Ok(files);
        }

        for entry in fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                files.push(path);
            }
        }

        files.sort();
        Ok(files)
    }

    /// 读取Netplan配置
    pub fn read_config(&self, file_path: &Path) -> Result<NetplanConfig> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("读取配置文件失败: {:?}", file_path))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("解析YAML配置失败: {:?}", file_path))
    }

    /// 写入Netplan配置
    pub fn write_config(&self, file_path: &Path, config: &NetplanConfig) -> Result<()> {
        let yaml = serde_yaml::to_string(config)
            .context("序列化配置失败")?;

        fs::write(file_path, yaml)
            .with_context(|| format!("写入配置文件失败: {:?}", file_path))
    }

    /// 备份配置文件
    pub fn backup_config(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = file_path.with_extension(format!("yaml.backup.{}", timestamp));

        fs::copy(file_path, &backup_path)
            .with_context(|| format!("备份配置文件失败: {:?}", file_path))?;

        println!("✅ 已备份配置到: {:?}", backup_path);
        Ok(backup_path)
    }

    /// 应用Netplan配置
    #[allow(dead_code)]
    pub fn apply(&self) -> Result<()> {
        let output = std::process::Command::new("netplan")
            .arg("apply")
            .output()
            .context("执行netplan apply失败")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("netplan apply失败: {}", stderr);
        }

        println!("✅ Netplan配置已应用");
        Ok(())
    }

    /// 测试Netplan配置（不实际应用）
    #[allow(dead_code)]
    pub fn try_config(&self) -> Result<()> {
        let output = std::process::Command::new("netplan")
            .arg("try")
            .arg("--timeout")
            .arg("10")
            .output()
            .context("执行netplan try失败")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("netplan try失败: {}", stderr);
        }

        Ok(())
    }

    /// 为接口设置静态IP
    pub fn set_static_ip(
        &self,
        iface_name: &str,
        address: &str,
        gateway: Option<&str>,
        nameservers: Option<Vec<String>>,
    ) -> Result<()> {
        // 查找或创建配置文件
        let config_file = self.find_or_create_config_file()?;

        // 备份原配置
        if config_file.exists() {
            self.backup_config(&config_file)?;
        }

        // 读取或创建配置
        let mut config = if config_file.exists() {
            self.read_config(&config_file)?
        } else {
            NetplanConfig::default()
        };

        // 设置接口配置
        let iface_config = InterfaceConfig {
            dhcp4: Some(false),
            dhcp6: Some(false),
            addresses: Some(vec![address.to_string()]),
            routes: gateway.map(|gw| {
                vec![RouteConfig {
                    to: "default".to_string(),
                    via: gw.to_string(),
                }]
            }),
            nameservers: nameservers.map(|ns| NameserverConfig { addresses: ns }),
            ..Default::default()
        };

        config.network.ethernets.insert(iface_name.to_string(), iface_config);

        // 写入配置
        self.write_config(&config_file, &config)?;

        println!("✅ 已更新Netplan配置: {:?}", config_file);
        Ok(())
    }

    /// 为接口设置DHCP
    pub fn set_dhcp(&self, iface_name: &str) -> Result<()> {
        let config_file = self.find_or_create_config_file()?;

        if config_file.exists() {
            self.backup_config(&config_file)?;
        }

        let mut config = if config_file.exists() {
            self.read_config(&config_file)?
        } else {
            NetplanConfig::default()
        };

        let iface_config = InterfaceConfig {
            dhcp4: Some(true),
            dhcp6: Some(false),
            ..Default::default()
        };

        config.network.ethernets.insert(iface_name.to_string(), iface_config);

        self.write_config(&config_file, &config)?;

        println!("✅ 已更新Netplan配置为DHCP: {:?}", config_file);
        Ok(())
    }

    /// 查找或创建配置文件
    fn find_or_create_config_file(&self) -> Result<PathBuf> {
        let files = self.list_config_files()?;

        if let Some(first_file) = files.first() {
            Ok(first_file.clone())
        } else {
            Ok(self.config_dir.join("01-netcfg.yaml"))
        }
    }
}

impl Default for NetplanManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Netplan配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetplanConfig {
    pub network: NetworkConfig,
}

impl Default for NetplanConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                version: 2,
                renderer: Some("networkd".to_string()),
                ethernets: HashMap::new(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer: Option<String>,
    #[serde(default)]
    pub ethernets: HashMap<String, InterfaceConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp4: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<RouteConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameservers: Option<NameserverConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub to: String,
    pub via: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameserverConfig {
    pub addresses: Vec<String>,
}

