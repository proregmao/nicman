// TUI界面模块 - 使用ratatui实现终端用户界面
use crate::backend::{owner_detection, runtime, traffic};
use crate::model::{InterfaceKind, InterfaceState, NetInterface};
use crate::utils::format::{format_bytes, format_speed};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::time::{Duration, Instant};

/// 应用状态
pub struct App {
    interfaces: Vec<NetInterface>,
    list_state: ListState,
    traffic_monitor: traffic::TrafficMonitor,
    last_update: Instant,
    screen: Screen,
    should_quit: bool,
    edit_form: Option<EditFormState>,  // 编辑表单状态
    action_menu_state: usize,  // 操作菜单选中项
}

/// 屏幕类型
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Main,
    Help,
    ConfirmDelete,  // 删除确认对话框
    EditIface,      // 编辑接口配置
    ToggleDhcp,     // 切换DHCP/静态确认
    OwnerActions,   // 创建者操作对话框
    InterfaceActions, // 接口操作菜单
}

/// 编辑表单状态
#[derive(Debug, Clone)]
struct EditFormState {
    interface_name: String,
    current_field: usize,  // 当前焦点字段
    is_editing: bool,      // 是否正在编辑字段
    ip_address: String,
    netmask: String,
    gateway: String,
    dns: String,
    error_message: Option<String>,
}

impl EditFormState {
    fn new(iface: &NetInterface) -> Self {
        // 从当前接口获取默认值
        let ip_address = iface.ipv4_addresses.first()
            .map(|addr| {
                // 提取IP地址部分（去掉/24这样的前缀）
                addr.split('/').next().unwrap_or("").to_string()
            })
            .unwrap_or_default();

        // 从ipv4_config读取子网掩码和网关
        let netmask = iface.ipv4_config.as_ref()
            .map(|cfg| cfg.netmask.clone())
            .unwrap_or_else(|| String::from("255.255.255.0"));

        let gateway = iface.ipv4_config.as_ref()
            .and_then(|cfg| cfg.gateway.clone())
            .unwrap_or_default();

        // 从dns_config读取DNS服务器
        let dns = iface.dns_config.as_ref()
            .map(|cfg| cfg.nameservers.join(","))
            .unwrap_or_else(|| String::from("223.5.5.5,114.114.114.114"));

        Self {
            interface_name: iface.name.clone(),
            current_field: 0,
            is_editing: false,
            ip_address,
            netmask,
            gateway,
            dns,
            error_message: None,
        }
    }

    fn field_count() -> usize {
        4  // IP、掩码、网关、DNS
    }

    fn next_field(&mut self) {
        self.current_field = (self.current_field + 1) % Self::field_count();
    }

    fn prev_field(&mut self) {
        if self.current_field == 0 {
            self.current_field = Self::field_count() - 1;
        } else {
            self.current_field -= 1;
        }
    }

    #[allow(dead_code)]
    fn current_field_value(&self) -> &str {
        match self.current_field {
            0 => &self.ip_address,
            1 => &self.netmask,
            2 => &self.gateway,
            3 => &self.dns,
            _ => "",
        }
    }

    fn current_field_value_mut(&mut self) -> &mut String {
        match self.current_field {
            0 => &mut self.ip_address,
            1 => &mut self.netmask,
            2 => &mut self.gateway,
            3 => &mut self.dns,
            _ => &mut self.ip_address,
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let interfaces = runtime::list_interfaces()?;
        let mut list_state = ListState::default();
        if !interfaces.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            interfaces,
            list_state,
            traffic_monitor: traffic::TrafficMonitor::new(),
            last_update: Instant::now(),
            screen: Screen::Main,
            should_quit: false,
            edit_form: None,
            action_menu_state: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key.code, key.modifiers)?;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick()?;
                last_tick = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match self.screen {
            Screen::Main => {
                match key {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Char('?') => self.screen = Screen::Help,
                    KeyCode::Char('r') => self.refresh()?,
                    KeyCode::Up | KeyCode::Char('k') => self.previous(),
                    KeyCode::Down | KeyCode::Char('j') => self.next(),
                    KeyCode::Enter => {
                        // 回车键：打开接口操作菜单
                        if self.list_state.selected().is_some() {
                            self.action_menu_state = 0;
                            self.screen = Screen::InterfaceActions;
                        }
                    }
                    KeyCode::Char('e') => {
                        // e键：快速编辑接口配置（仅物理接口）
                        if let Some(i) = self.list_state.selected() {
                            if let Some(iface) = self.interfaces.get(i) {
                                if matches!(iface.kind, InterfaceKind::Physical) {
                                    self.edit_form = Some(EditFormState::new(iface));
                                    self.screen = Screen::EditIface;
                                }
                            }
                        }
                    }
                    KeyCode::Char('t') => {
                        // 切换DHCP/静态（仅物理接口）
                        if let Some(i) = self.list_state.selected() {
                            if let Some(iface) = self.interfaces.get(i) {
                                if matches!(iface.kind, InterfaceKind::Physical) {
                                    self.screen = Screen::ToggleDhcp;
                                }
                            }
                        }
                    }
                    KeyCode::Char('x') | KeyCode::Delete => {
                        // 删除接口（仅虚拟接口）
                        if let Some(i) = self.list_state.selected() {
                            if let Some(iface) = self.interfaces.get(i) {
                                if iface.kind != InterfaceKind::Physical && iface.kind != InterfaceKind::Loopback {
                                    self.screen = Screen::ConfirmDelete;
                                }
                            }
                        }
                    }
                    KeyCode::Char('u') => {
                        // 启用接口 (up)
                        self.toggle_interface_up()?;
                    }
                    KeyCode::Char('d') => {
                        // 禁用接口 (down)
                        self.toggle_interface_down()?;
                    }
                    KeyCode::Char('o') => {
                        // 创建者操作（停止服务/容器/进程等）
                        if let Some(i) = self.list_state.selected() {
                            if let Some(iface) = self.interfaces.get(i) {
                                if iface.owner.is_some() {
                                    self.screen = Screen::OwnerActions;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Screen::Help => {
                if matches!(key, KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?')) {
                    self.screen = Screen::Main;
                }
            }
            Screen::OwnerActions => {
                match key {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        // 确认执行（Y键或Enter键）
                        self.execute_owner_action()?;
                        self.screen = Screen::Main;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                        // 取消（N键、Esc键或q键）
                        self.screen = Screen::Main;
                    }
                    _ => {}
                }
            }
            Screen::InterfaceActions => {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.action_menu_state > 0 {
                            self.action_menu_state -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max_items = self.get_action_menu_items().len();
                        if self.action_menu_state < max_items.saturating_sub(1) {
                            self.action_menu_state += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.execute_action_menu_item()?;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // 退出菜单（Esc键或q键）
                        self.screen = Screen::Main;
                    }
                    _ => {}
                }
            }
            Screen::EditIface => {
                self.handle_edit_form_key(key)?;
            }
            Screen::ToggleDhcp => {
                match key {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        // 确认切换到DHCP（Y键或Enter键）
                        self.toggle_dhcp()?;
                        self.screen = Screen::Main;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                        // 取消（N键、Esc键或q键）
                        self.screen = Screen::Main;
                    }
                    _ => {}
                }
            }
            Screen::ConfirmDelete => {
                match key {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        // 确认删除（Y键或Enter键）
                        self.delete_selected_interface()?;
                        self.screen = Screen::Main;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                        // 取消删除（N键、Esc键或q键）
                        self.screen = Screen::Main;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_edit_form_key(&mut self, key: KeyCode) -> Result<()> {
        if let Some(form) = &mut self.edit_form {
            if form.is_editing {
                // 正在编辑字段内容
                match key {
                    KeyCode::Esc => {
                        // 退出编辑模式
                        form.is_editing = false;
                    }
                    KeyCode::Enter => {
                        // 完成编辑，返回导航模式
                        form.is_editing = false;
                    }
                    KeyCode::Backspace => {
                        // 删除字符
                        let value = form.current_field_value_mut();
                        value.pop();
                    }
                    KeyCode::Char(c) => {
                        // 输入字符
                        let value = form.current_field_value_mut();
                        value.push(c);
                    }
                    _ => {}
                }
            } else {
                // 导航模式
                match key {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        // 取消编辑，返回主界面（Esc键或q键）
                        self.edit_form = None;
                        self.screen = Screen::Main;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        // 上一个字段
                        form.prev_field();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        // 下一个字段
                        form.next_field();
                    }
                    KeyCode::Enter => {
                        // 进入编辑模式
                        form.is_editing = true;
                    }
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        // 保存配置
                        if let Err(e) = self.save_interface_config() {
                            if let Some(form) = &mut self.edit_form {
                                form.error_message = Some(format!("保存失败: {}", e));
                            }
                        } else {
                            self.edit_form = None;
                            self.screen = Screen::Main;
                            self.refresh()?;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn on_tick(&mut self) -> Result<()> {
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            self.traffic_monitor.update_all(&mut self.interfaces)?;
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn refresh(&mut self) -> Result<()> {
        self.interfaces = runtime::list_interfaces()?;
        for iface in &mut self.interfaces {
            iface.owner = owner_detection::OwnerDetector::detect(iface);
        }
        self.traffic_monitor.update_all(&mut self.interfaces)?;
        Ok(())
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.interfaces.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.interfaces.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_interface_up(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                runtime::set_interface_up(&iface.name)?;
                self.refresh()?;
            }
        }
        Ok(())
    }

    fn toggle_interface_down(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                runtime::set_interface_down(&iface.name)?;
                self.refresh()?;
            }
        }
        Ok(())
    }

    fn save_interface_config(&mut self) -> Result<()> {
        if let Some(form) = &self.edit_form {
            let iface_name = &form.interface_name;

            // 验证输入
            if form.ip_address.is_empty() {
                return Err(anyhow::anyhow!("IP地址不能为空"));
            }
            if form.gateway.is_empty() {
                return Err(anyhow::anyhow!("网关不能为空"));
            }

            // 将子网掩码转换为前缀长度
            let prefix = Self::netmask_to_prefix(&form.netmask)?;

            // 1. 运行时修改（立即生效）
            runtime::flush_ipv4_addresses(iface_name)?;
            runtime::set_ipv4_address(iface_name, &form.ip_address, prefix)?;
            runtime::set_default_gateway(&form.gateway, iface_name)?;

            // 2. 持久化到Netplan
            use crate::backend::netplan::NetplanManager;
            let netplan = NetplanManager::new();

            // 解析DNS列表
            let dns_list: Vec<String> = form.dns
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            netplan.set_static_ip(
                iface_name,
                &format!("{}/{}", form.ip_address, prefix),
                Some(&form.gateway),
                Some(dns_list),
            )?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("编辑表单状态丢失"))
        }
    }

    fn toggle_dhcp(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                use crate::backend::netplan::NetplanManager;
                let netplan = NetplanManager::new();
                netplan.set_dhcp(&iface.name)?;
            }
        }
        Ok(())
    }

    fn netmask_to_prefix(netmask: &str) -> Result<u8> {
        let parts: Vec<u8> = netmask
            .split('.')
            .map(|s| s.parse::<u8>())
            .collect::<Result<Vec<_>, _>>()?;

        if parts.len() != 4 {
            return Err(anyhow::anyhow!("无效的子网掩码格式"));
        }

        let mask = ((parts[0] as u32) << 24)
            | ((parts[1] as u32) << 16)
            | ((parts[2] as u32) << 8)
            | (parts[3] as u32);

        Ok(mask.count_ones() as u8)
    }

    fn delete_selected_interface(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i).cloned() {
                // 使用智能删除
                use crate::backend::removal::RemovalManager;
                let strategy = RemovalManager::determine_strategy(&iface);
                RemovalManager::remove_interface(&iface, &strategy)?;
                self.refresh()?;

                // 调整选中项
                if self.interfaces.is_empty() {
                    self.list_state.select(None);
                } else if i >= self.interfaces.len() {
                    self.list_state.select(Some(self.interfaces.len() - 1));
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        match self.screen {
            Screen::Main => self.draw_main(f),
            Screen::Help => self.draw_help(f),
            Screen::EditIface => {
                self.draw_main(f);
                self.draw_edit_form(f);
            }
            Screen::ToggleDhcp => {
                self.draw_main(f);
                self.draw_toggle_dhcp(f);
            }
            Screen::ConfirmDelete => {
                self.draw_main(f);
                self.draw_confirm_delete(f);
            }
            Screen::OwnerActions => {
                self.draw_main(f);
                self.draw_owner_actions(f);
            }
            Screen::InterfaceActions => {
                self.draw_main(f);
                self.draw_interface_actions(f);
            }
        }
    }

    fn draw_main(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(f.size());

        self.draw_interface_list(f, chunks[0]);
        self.draw_details(f, chunks[1]);
    }

    fn draw_interface_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .interfaces
            .iter()
            .map(|iface| {
                let icon = match iface.kind {
                    InterfaceKind::Physical => "🔌",
                    InterfaceKind::Loopback => "🔄",
                    InterfaceKind::Docker => "🐳",
                    InterfaceKind::WireGuard => "🔐",
                    InterfaceKind::Bridge => "🌉",
                    InterfaceKind::Veth => "🔗",
                    InterfaceKind::Vlan => "📡",
                    InterfaceKind::Tun => "🚇",
                    InterfaceKind::Tap => "🚰",
                    InterfaceKind::Unknown => "❓",
                };

                let state_icon = match iface.state {
                    InterfaceState::Up => "✅",
                    InterfaceState::Down => "❌",
                    InterfaceState::Unknown => "❓",
                };

                let speed_info = format!(
                    "↓ {} ↑ {}",
                    format_speed(iface.traffic_stats.rx_speed),
                    format_speed(iface.traffic_stats.tx_speed)
                );

                let content = format!("{} {} {} - {}", icon, state_icon, iface.name, speed_info);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("网络接口 (↑↓:选择 r:刷新 q:退出 ?:帮助)")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn draw_details(&self, f: &mut Frame, area: Rect) {
        let selected = self.list_state.selected();

        if let Some(i) = selected {
            if let Some(iface) = self.interfaces.get(i) {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(area);

                self.draw_interface_info(f, chunks[0], iface);
                self.draw_traffic_stats(f, chunks[1], iface);
            }
        }
    }

    fn draw_interface_info(&self, f: &mut Frame, area: Rect, iface: &NetInterface) {
        let mut lines = vec![
            Line::from(vec![
                Span::styled("接口名称: ", Style::default().fg(Color::Cyan)),
                Span::raw(&iface.name),
            ]),
            Line::from(vec![
                Span::styled("类型: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:?}", iface.kind)),
            ]),
            Line::from(vec![
                Span::styled("状态: ", Style::default().fg(Color::Cyan)),
                Span::raw(format!("{:?}", iface.state)),
            ]),
        ];

        if let Some(mac) = &iface.mac_address {
            lines.push(Line::from(vec![
                Span::styled("MAC地址: ", Style::default().fg(Color::Cyan)),
                Span::raw(mac),
            ]));
        }

        if !iface.ipv4_addresses.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("IPv4地址: ", Style::default().fg(Color::Cyan)),
                Span::raw(iface.ipv4_addresses.join(", ")),
            ]));
        }

        // 显示子网掩码
        if let Some(ipv4_config) = &iface.ipv4_config {
            lines.push(Line::from(vec![
                Span::styled("子网掩码: ", Style::default().fg(Color::Cyan)),
                Span::raw(&ipv4_config.netmask),
            ]));

            // 显示网关
            if let Some(gateway) = &ipv4_config.gateway {
                lines.push(Line::from(vec![
                    Span::styled("网关: ", Style::default().fg(Color::Cyan)),
                    Span::raw(gateway),
                ]));
            }
        }

        // 显示DNS
        if let Some(dns_config) = &iface.dns_config {
            if !dns_config.nameservers.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("DNS: ", Style::default().fg(Color::Cyan)),
                    Span::raw(dns_config.nameservers.join(",")),
                ]));
            }
        }

        if !iface.ipv6_addresses.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("IPv6地址: ", Style::default().fg(Color::Cyan)),
                Span::raw(iface.ipv6_addresses.join(", ")),
            ]));
        }

        if let Some(owner) = &iface.owner {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("创建者: ", Style::default().fg(Color::Yellow)),
                Span::raw(owner.display_name()),
            ]));

            // 显示详细信息和操作提示
            use crate::model::InterfaceOwner;
            match owner {
                InterfaceOwner::SystemdService { name, status, .. } => {
                    lines.push(Line::from(vec![
                        Span::styled("  服务名: ", Style::default().fg(Color::Cyan)),
                        Span::raw(name),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  状态: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{:?}", status)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  操作: ", Style::default().fg(Color::Green)),
                        Span::raw("按 'o' 键停止服务"),
                    ]));
                },
                InterfaceOwner::DockerContainer { id, name, image } => {
                    lines.push(Line::from(vec![
                        Span::styled("  容器ID: ", Style::default().fg(Color::Cyan)),
                        Span::raw(&id[..12.min(id.len())]),  // 显示前12位
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  容器名: ", Style::default().fg(Color::Cyan)),
                        Span::raw(name),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  镜像: ", Style::default().fg(Color::Cyan)),
                        Span::raw(image),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  操作: ", Style::default().fg(Color::Green)),
                        Span::raw("按 'o' 键停止容器"),
                    ]));
                },
                InterfaceOwner::Process { pid, name, cmdline } => {
                    lines.push(Line::from(vec![
                        Span::styled("  进程ID: ", Style::default().fg(Color::Cyan)),
                        Span::raw(format!("{}", pid)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  进程名: ", Style::default().fg(Color::Cyan)),
                        Span::raw(name),
                    ]));
                    if !cmdline.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled("  命令行: ", Style::default().fg(Color::Cyan)),
                            Span::raw(cmdline),
                        ]));
                    }
                    lines.push(Line::from(vec![
                        Span::styled("  操作: ", Style::default().fg(Color::Green)),
                        Span::raw("按 'o' 键终止进程"),
                    ]));
                },
                InterfaceOwner::NetworkManager { connection, .. } => {
                    lines.push(Line::from(vec![
                        Span::styled("  连接名: ", Style::default().fg(Color::Cyan)),
                        Span::raw(connection),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  操作: ", Style::default().fg(Color::Green)),
                        Span::raw("按 'o' 键断开连接"),
                    ]));
                },
                InterfaceOwner::Kernel { module } => {
                    lines.push(Line::from(vec![
                        Span::styled("  内核模块: ", Style::default().fg(Color::Cyan)),
                        Span::raw(module),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("  操作: ", Style::default().fg(Color::Green)),
                        Span::raw("按 'o' 键卸载模块"),
                    ]));
                },
                InterfaceOwner::Unknown => {},
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("接口详情")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn draw_traffic_stats(&self, f: &mut Frame, area: Rect, iface: &NetInterface) {
        let stats = &iface.traffic_stats;

        let lines = vec![
            Line::from(vec![
                Span::styled("接收: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{} ({} 包)", format_bytes(stats.rx_bytes), stats.rx_packets)),
            ]),
            Line::from(vec![
                Span::styled("发送: ", Style::default().fg(Color::Blue)),
                Span::raw(format!("{} ({} 包)", format_bytes(stats.tx_bytes), stats.tx_packets)),
            ]),
            Line::from(vec![
                Span::styled("速率: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("↓ {}  ↑ {}", format_speed(stats.rx_speed), format_speed(stats.tx_speed))),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title("流量统计")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            );

        f.render_widget(paragraph, area);
    }

    fn draw_help(&self, f: &mut Frame) {
        let help_text = vec![
            Line::from(Span::styled("网卡管理工具 - 帮助", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("导航:", Style::default().fg(Color::Cyan))),
            Line::from("  ↑/k      - 上移"),
            Line::from("  ↓/j      - 下移"),
            Line::from(""),
            Line::from(Span::styled("物理接口操作:", Style::default().fg(Color::Cyan))),
            Line::from("  Enter/e  - 编辑IP/掩码/网关/DNS"),
            Line::from("  t        - 切换DHCP/静态模式"),
            Line::from("  u        - 启用接口 (Up)"),
            Line::from("  d        - 禁用接口 (Down)"),
            Line::from(""),
            Line::from(Span::styled("虚拟接口操作:", Style::default().fg(Color::Cyan))),
            Line::from("  x/Del    - 删除接口"),
            Line::from("  u        - 启用接口 (Up)"),
            Line::from("  d        - 禁用接口 (Down)"),
            Line::from(""),
            Line::from(Span::styled("创建者操作:", Style::default().fg(Color::Cyan))),
            Line::from("  o        - 停止服务/容器/进程"),
            Line::from("             (停止systemd服务)"),
            Line::from("             (停止Docker容器)"),
            Line::from("             (终止进程)"),
            Line::from("             (断开NetworkManager连接)"),
            Line::from("             (卸载内核模块)"),
            Line::from(""),
            Line::from(Span::styled("通用操作:", Style::default().fg(Color::Cyan))),
            Line::from("  r        - 刷新接口列表"),
            Line::from("  q        - 退出程序"),
            Line::from("  ?        - 显示/隐藏帮助"),
            Line::from(""),
            Line::from(Span::styled("编辑表单:", Style::default().fg(Color::Cyan))),
            Line::from("  Tab      - 下一个字段"),
            Line::from("  Shift+Tab- 上一个字段"),
            Line::from("  Enter    - 保存配置"),
            Line::from("  Esc      - 取消编辑"),
            Line::from(""),
            Line::from(Span::styled("确认对话框:", Style::default().fg(Color::Cyan))),
            Line::from("  Y        - 确认操作"),
            Line::from("  N/Esc    - 取消操作"),
            Line::from(""),
            Line::from(Span::styled("按任意键返回", Style::default().fg(Color::Green))),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title("帮助")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
            )
            .alignment(Alignment::Left);

        let area = centered_rect(60, 60, f.size());
        f.render_widget(paragraph, area);
    }

    fn draw_confirm_delete(&self, f: &mut Frame) {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                // 计算弹窗区域
                let area = centered_rect(60, 50, f.size());

                // 只清除弹窗区域
                f.render_widget(Clear, area);
                use crate::backend::removal::RemovalManager;
                let strategy = RemovalManager::determine_strategy(iface);
                let warnings = RemovalManager::check_safety(iface);

                let mut text = vec![
                    Line::from(Span::styled(
                        "确认删除接口",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("接口名称: "),
                        Span::styled(&iface.name, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::raw("接口类型: "),
                        Span::raw(format!("{:?}", iface.kind)),
                    ]),
                    Line::from(vec![
                        Span::raw("删除策略: "),
                        Span::styled(
                            format!("{:?}", strategy),
                            Style::default().fg(Color::Cyan),
                        ),
                    ]),
                    Line::from(""),
                ];

                // 显示警告
                if !warnings.is_empty() {
                    text.push(Line::from(Span::styled(
                        "⚠️  警告:",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )));
                    for warning in &warnings {
                        text.push(Line::from(Span::styled(
                            format!("  • {}", warning),
                            Style::default().fg(Color::Yellow),
                        )));
                    }
                    text.push(Line::from(""));
                }

                text.push(Line::from(Span::styled(
                    "确定要删除此接口吗？",
                    Style::default().fg(Color::Red),
                )));
                text.push(Line::from(""));
                text.push(Line::from(vec![
                    Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" - 确认删除  "),
                    Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(" - 取消"),
                ]));

                let paragraph = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("删除确认")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Red))
                            .style(Style::default().bg(Color::Black)),
                    )
                    .alignment(Alignment::Left);

                // area已经在前面计算过了
                f.render_widget(paragraph, area);
            }
        }
    }

    fn draw_edit_form(&self, f: &mut Frame) {
        if let Some(form) = &self.edit_form {
            // 计算弹窗区域
            let area = centered_rect(70, 60, f.size());

            // 只清除弹窗区域
            f.render_widget(Clear, area);

            let field_names = ["IP地址", "子网掩码", "网关", "DNS"];
            let field_values = [
                &form.ip_address,
                &form.netmask,
                &form.gateway,
                &form.dns,
            ];

            let mut text = vec![
                Line::from(Span::styled(
                    format!("编辑接口配置 - {}", form.interface_name),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            // 显示表单字段
            for (i, (name, value)) in field_names.iter().zip(field_values.iter()).enumerate() {
                let is_current = i == form.current_field;
                let is_editing_this = is_current && form.is_editing;

                let style = if is_editing_this {
                    // 正在编辑：青色背景，黑色文字
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if is_current {
                    // 当前选中但未编辑：深灰背景，青色文字
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
                } else {
                    // 未选中：白色文字
                    Style::default().fg(Color::White)
                };

                let cursor = if is_editing_this {
                    "✎ "  // 编辑图标
                } else if is_current {
                    "► "  // 选中图标
                } else {
                    "  "  // 空格
                };

                text.push(Line::from(vec![
                    Span::styled(
                        cursor,
                        Style::default().fg(if is_editing_this { Color::Yellow } else { Color::Green }),
                    ),
                    Span::styled(format!("{:12}: ", name), style),
                    Span::styled(*value, style),
                ]));
            }

            text.push(Line::from(""));

            // 显示错误信息
            if let Some(err) = &form.error_message {
                text.push(Line::from(Span::styled(
                    format!("❌ {}", err),
                    Style::default().fg(Color::Red),
                )));
                text.push(Line::from(""));
            }

            text.push(Line::from(""));

            // 根据模式显示不同的操作提示
            if form.is_editing {
                text.push(Line::from(Span::styled(
                    "编辑模式:",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                text.push(Line::from("  输入字符 - 编辑内容"));
                text.push(Line::from("  Backspace - 删除字符"));
                text.push(Line::from("  Enter - 完成编辑"));
                text.push(Line::from("  Esc - 取消编辑"));
            } else {
                text.push(Line::from(Span::styled(
                    "导航模式:",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                text.push(Line::from("  ↑/↓ 或 k/j - 切换字段"));
                text.push(Line::from("  Enter - 编辑当前字段"));
                text.push(Line::from("  s - 保存配置"));
                text.push(Line::from("  Esc - 取消"));
            }

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .title("编辑配置")
                        .style(Style::default().bg(Color::Black))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .alignment(Alignment::Left);

            // area已经在前面计算过了
            f.render_widget(paragraph, area);
        }
    }

    fn draw_toggle_dhcp(&self, f: &mut Frame) {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                // 计算弹窗区域
                let area = centered_rect(60, 50, f.size());

                // 只清除弹窗区域
                f.render_widget(Clear, area);
                let text = vec![
                    Line::from(Span::styled(
                        "切换到DHCP模式",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("接口名称: "),
                        Span::styled(&iface.name, Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "⚠️  警告:",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from("  • 当前静态IP配置将被清除"),
                    Line::from("  • 接口将自动从DHCP服务器获取IP"),
                    Line::from("  • 此操作将修改Netplan配置"),
                    Line::from(""),
                    Line::from(Span::styled(
                        "确定要切换到DHCP模式吗？",
                        Style::default().fg(Color::Yellow),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::raw(" - 确认切换  "),
                        Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                        Span::raw(" - 取消"),
                    ]),
                ];

                let paragraph = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("切换DHCP")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Yellow))
                            .style(Style::default().bg(Color::Black)),
                    )
                    .alignment(Alignment::Left);

                // area已经在前面计算过了
                f.render_widget(paragraph, area);
            }
        }
    }

    fn draw_owner_actions(&self, f: &mut Frame) {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                if let Some(owner) = &iface.owner {
                    // 计算弹窗区域
                    let area = centered_rect(70, 60, f.size());

                    // 只清除弹窗区域
                    f.render_widget(Clear, area);

                    use crate::model::InterfaceOwner;
                    let (action_name, action_desc, warning) = match owner {
                        InterfaceOwner::SystemdService { name, .. } => (
                            "停止systemd服务",
                            format!("服务名: {}\n\n将执行: systemctl stop {}", name, name),
                            "⚠️ 警告：停止服务可能影响系统功能！",
                        ),
                        InterfaceOwner::DockerContainer { id, name, .. } => (
                            "停止Docker容器",
                            format!("容器名: {}\n容器ID: {}\n\n将执行: docker stop {}", name, &id[..12.min(id.len())], &id[..12.min(id.len())]),
                            "⚠️ 警告：停止容器将中断容器内的所有服务！",
                        ),
                        InterfaceOwner::Process { pid, name, .. } => (
                            "终止进程",
                            format!("进程名: {}\n进程ID: {}\n\n将执行: kill {}", name, pid, pid),
                            "⚠️ 警告：强制终止进程可能导致数据丢失！",
                        ),
                        InterfaceOwner::NetworkManager { connection, .. } => (
                            "断开NetworkManager连接",
                            format!("连接名: {}\n\n将执行: nmcli connection down {}", connection, connection),
                            "⚠️ 警告：断开连接将中断网络服务！",
                        ),
                        InterfaceOwner::Kernel { module } => (
                            "卸载内核模块",
                            format!("模块名: {}\n\n将执行: rmmod {}", module, module),
                            "⚠️ 警告：卸载内核模块可能导致系统不稳定！",
                        ),
                        InterfaceOwner::Unknown => return,
                    };

                    let text = vec![
                        Line::from(Span::styled(
                            action_name,
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                        )),
                        Line::from(""),
                        Line::from(Span::styled(warning, Style::default().fg(Color::Red))),
                        Line::from(""),
                        Line::from(action_desc),
                        Line::from(""),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                            Span::raw(" - 确认执行  "),
                            Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                            Span::raw(" - 取消"),
                        ]),
                    ];

                    let paragraph = Paragraph::new(text)
                        .block(
                            Block::default()
                                .title("创建者操作")
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .border_style(Style::default().fg(Color::Yellow))
                                .style(Style::default().bg(Color::Black)),
                        )
                        .alignment(Alignment::Left);

                    f.render_widget(paragraph, area);
                }
            }
        }
    }

    fn execute_owner_action(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                if let Some(owner) = &iface.owner {
                    use crate::model::InterfaceOwner;
                    use crate::utils::command::execute_command_stdout;

                    let result = match owner {
                        InterfaceOwner::SystemdService { name, .. } => {
                            execute_command_stdout("systemctl", &["stop", name])
                        },
                        InterfaceOwner::DockerContainer { id, .. } => {
                            // 检查是否是系统网桥（docker0等）
                            if id == "system" {
                                // docker0是系统网桥，不能通过docker stop停止
                                // 返回一个友好的错误信息
                                return Err(anyhow::anyhow!("Docker网桥是系统组件，无法停止。请使用 'systemctl stop docker' 停止Docker服务。"));
                            }
                            execute_command_stdout("docker", &["stop", id])
                        },
                        InterfaceOwner::Process { pid, .. } => {
                            execute_command_stdout("kill", &[&pid.to_string()])
                        },
                        InterfaceOwner::NetworkManager { connection, .. } => {
                            execute_command_stdout("nmcli", &["connection", "down", connection])
                        },
                        InterfaceOwner::Kernel { module } => {
                            execute_command_stdout("rmmod", &[module])
                        },
                        InterfaceOwner::Unknown => return Ok(()),
                    };

                    // 等待一下让操作生效
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    // 刷新接口列表
                    self.refresh()?;

                    // 检查操作结果，如果失败则显示错误但不退出程序
                    if let Err(e) = result {
                        eprintln!("操作失败: {}", e);
                        // 不传播错误，避免程序退出
                    }
                }
            }
        }
        Ok(())
    }

    fn get_action_menu_items(&self) -> Vec<(&str, &str)> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                let mut items = Vec::new();

                // 物理接口的操作
                if matches!(iface.kind, InterfaceKind::Physical) {
                    items.push(("编辑配置", "修改IP/掩码/网关/DNS"));
                    items.push(("切换DHCP", "切换DHCP/静态模式"));
                    items.push(("启用接口", "设置接口状态为UP"));
                    items.push(("禁用接口", "设置接口状态为DOWN"));
                }

                // 虚拟接口的操作
                if iface.kind != InterfaceKind::Physical && iface.kind != InterfaceKind::Loopback {
                    items.push(("删除接口", "删除虚拟网络接口"));
                    items.push(("启用接口", "设置接口状态为UP"));
                    items.push(("禁用接口", "设置接口状态为DOWN"));
                }

                // 如果有创建者，添加创建者操作
                if let Some(owner) = &iface.owner {
                    use crate::model::InterfaceOwner;
                    match owner {
                        InterfaceOwner::SystemdService { .. } => {
                            items.push(("停止服务", "停止systemd服务"));
                        },
                        InterfaceOwner::DockerContainer { id, .. } => {
                            // 只有真实的容器才显示"停止容器"选项
                            // docker0等系统网桥的id是"system"，不显示停止选项
                            if id != "system" {
                                items.push(("停止容器", "停止Docker容器"));
                            }
                        },
                        InterfaceOwner::Process { .. } => {
                            items.push(("终止进程", "终止创建者进程"));
                        },
                        InterfaceOwner::NetworkManager { .. } => {
                            items.push(("断开连接", "断开NetworkManager连接"));
                        },
                        InterfaceOwner::Kernel { .. } => {
                            items.push(("卸载模块", "卸载内核模块"));
                        },
                        InterfaceOwner::Unknown => {},
                    }
                }

                return items;
            }
        }
        Vec::new()
    }

    fn draw_interface_actions(&self, f: &mut Frame) {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i) {
                let area = centered_rect(60, 70, f.size());
                f.render_widget(Clear, area);

                let items = self.get_action_menu_items();
                let mut text = vec![
                    Line::from(Span::styled(
                        format!("接口操作 - {}", iface.name),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                ];

                // 显示接口基本信息
                text.push(Line::from(vec![
                    Span::styled("接口类型: ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{:?}", iface.kind)),
                ]));

                // 显示创建者信息
                if let Some(owner) = &iface.owner {
                    text.push(Line::from(vec![
                        Span::styled("创建者: ", Style::default().fg(Color::Cyan)),
                        Span::raw(owner.display_name()),
                    ]));
                }

                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    "可用操作:",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
                text.push(Line::from(""));

                // 显示操作菜单
                for (idx, (action, desc)) in items.iter().enumerate() {
                    let prefix = if idx == self.action_menu_state {
                        "► "
                    } else {
                        "  "
                    };

                    let style = if idx == self.action_menu_state {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    text.push(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(*action, style),
                        Span::raw(" - "),
                        Span::styled(*desc, Style::default().fg(Color::DarkGray)),
                    ]));
                }

                text.push(Line::from(""));
                text.push(Line::from(""));
                text.push(Line::from(vec![
                    Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                    Span::raw(" - 选择  "),
                    Span::styled("Enter", Style::default().fg(Color::Green)),
                    Span::raw(" - 执行  "),
                    Span::styled("Esc", Style::default().fg(Color::Red)),
                    Span::raw(" - 取消"),
                ]));

                let paragraph = Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("接口操作菜单")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Cyan))
                            .style(Style::default().bg(Color::Black)),
                    )
                    .alignment(Alignment::Left);

                f.render_widget(paragraph, area);
            }
        }
    }

    fn execute_action_menu_item(&mut self) -> Result<()> {
        if let Some(i) = self.list_state.selected() {
            if let Some(iface) = self.interfaces.get(i).cloned() {
                let items = self.get_action_menu_items();
                if let Some((action, _)) = items.get(self.action_menu_state) {
                    match *action {
                        "编辑配置" => {
                            self.edit_form = Some(EditFormState::new(&iface));
                            self.screen = Screen::EditIface;
                        },
                        "切换DHCP" => {
                            self.screen = Screen::ToggleDhcp;
                        },
                        "启用接口" => {
                            self.screen = Screen::Main;
                            self.toggle_interface_up()?;
                        },
                        "禁用接口" => {
                            self.screen = Screen::Main;
                            self.toggle_interface_down()?;
                        },
                        "删除接口" => {
                            self.screen = Screen::ConfirmDelete;
                        },
                        "停止服务" | "停止容器" | "终止进程" | "断开连接" | "卸载模块" => {
                            self.screen = Screen::OwnerActions;
                        },
                        _ => {
                            self.screen = Screen::Main;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
