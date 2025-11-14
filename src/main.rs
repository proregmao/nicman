// 网卡管理工具主程序
mod model;
mod backend;
mod ui;
mod utils;

use clap::Parser;
use std::process;

/// 网卡管理工具 - TUI终端界面
#[derive(Parser, Debug)]
#[command(name = "nicman")]
#[command(about = "Linux网络接口管理工具", long_about = None)]
struct Args {
    /// 显示版本信息
    #[arg(short, long)]
    version: bool,
}

fn main() {
    let args = Args::parse();

    if args.version {
        println!("nicman v0.1.0");
        println!("Linux网络接口管理工具");
        return;
    }

    // 检查root权限
    if !is_root() {
        eprintln!("错误: 此程序需要root权限运行");
        eprintln!("请使用: sudo nicman");
        process::exit(1);
    }

    // 运行TUI应用
    match ui::App::new() {
        Ok(mut app) => {
            if let Err(e) = app.run() {
                eprintln!("应用运行错误: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("初始化失败: {}", e);
            process::exit(1);
        }
    }
}

/// 检查是否以root权限运行
fn is_root() -> bool {
    use nix::unistd::Uid;
    Uid::effective().is_root()
}
