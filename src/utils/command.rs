// 命令执行工具

use anyhow::{Context, Result};
use std::process::{Command, Output};

/// 执行系统命令并返回输出
pub fn execute_command(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("执行命令失败: {} {}", program, args.join(" ")))
}

/// 执行命令并返回stdout字符串
pub fn execute_command_stdout(program: &str, args: &[&str]) -> Result<String> {
    let output = execute_command(program, args)?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("命令执行失败: {}", stderr);
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 检查命令是否执行成功
pub fn command_success(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

