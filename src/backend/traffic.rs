// 流量监控模块 - 读取/sys/class/net统计数据，计算实时速率
use crate::model::{NetInterface, TrafficStats};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

/// 流量监控器
pub struct TrafficMonitor {
    stats_cache: HashMap<String, TrafficStats>,
    #[allow(dead_code)]
    update_interval: Duration,
}

impl TrafficMonitor {
    /// 创建新的流量监控器
    pub fn new() -> Self {
        Self {
            stats_cache: HashMap::new(),
            update_interval: Duration::from_secs(1),
        }
    }

    /// 更新所有接口的流量统计
    pub fn update_all(&mut self, interfaces: &mut [NetInterface]) -> Result<()> {
        for iface in interfaces {
            self.update_interface(iface)?;
        }
        Ok(())
    }

    /// 更新单个接口的流量统计
    pub fn update_interface(&mut self, iface: &mut NetInterface) -> Result<()> {
        let new_stats = self.read_stats(&iface.name)?;

        // 如果有缓存的旧数据，计算速率
        if let Some(old_stats) = self.stats_cache.get(&iface.name) {
            let duration = new_stats.last_update.duration_since(old_stats.last_update);
            let secs = duration.as_secs_f64();

            if secs > 0.0 {
                let mut updated_stats = new_stats.clone();
                updated_stats.rx_speed = (new_stats.rx_bytes.saturating_sub(old_stats.rx_bytes)) as f64 / secs;
                updated_stats.tx_speed = (new_stats.tx_bytes.saturating_sub(old_stats.tx_bytes)) as f64 / secs;

                iface.traffic_stats = updated_stats.clone();
                self.stats_cache.insert(iface.name.clone(), updated_stats);
            } else {
                iface.traffic_stats = new_stats.clone();
                self.stats_cache.insert(iface.name.clone(), new_stats);
            }
        } else {
            // 第一次读取，没有速率数据
            iface.traffic_stats = new_stats.clone();
            self.stats_cache.insert(iface.name.clone(), new_stats);
        }

        Ok(())
    }

    /// 从/sys/class/net读取接口统计数据
    fn read_stats(&self, iface_name: &str) -> Result<TrafficStats> {
        let base_path = format!("/sys/class/net/{}/statistics", iface_name);

        let rx_bytes = read_stat_file(&format!("{}/rx_bytes", base_path))?;
        let tx_bytes = read_stat_file(&format!("{}/tx_bytes", base_path))?;
        let rx_packets = read_stat_file(&format!("{}/rx_packets", base_path))?;
        let tx_packets = read_stat_file(&format!("{}/tx_packets", base_path))?;
        let rx_errors = read_stat_file(&format!("{}/rx_errors", base_path))?;
        let tx_errors = read_stat_file(&format!("{}/tx_errors", base_path))?;
        let rx_dropped = read_stat_file(&format!("{}/rx_dropped", base_path))?;
        let tx_dropped = read_stat_file(&format!("{}/tx_dropped", base_path))?;

        Ok(TrafficStats {
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
            rx_dropped,
            tx_dropped,
            rx_speed: 0.0,
            tx_speed: 0.0,
            last_update: Instant::now(),
        })
    }
}

impl Default for TrafficMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 读取统计文件中的数值
fn read_stat_file(path: &str) -> Result<u64> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("读取统计文件失败: {}", path))?;

    content.trim()
        .parse::<u64>()
        .with_context(|| format!("解析统计数据失败: {}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InterfaceKind;

    #[test]
    fn test_traffic_monitor_creation() {
        let monitor = TrafficMonitor::new();
        assert_eq!(monitor.stats_cache.len(), 0);
    }

    #[test]
    fn test_read_stats_lo() {
        // 测试读取lo接口的统计数据
        let monitor = TrafficMonitor::new();
        if let Ok(stats) = monitor.read_stats("lo") {
            assert!(stats.rx_bytes > 0 || stats.tx_bytes > 0);
        }
    }
}

