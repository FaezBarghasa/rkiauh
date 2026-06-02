use std::sync::{Arc, Mutex};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, Cell, BorderType, Clear},
    Frame,
};
use crate::installer::systemd::ServiceStatus;

pub struct App {
    pub r_klipp_status: ServiceStatus,
    pub rusted_moonraker_status: ServiceStatus,
    pub r_klipper_screen_status: ServiceStatus,
    pub fluidd_status: String,
    pub mainsail_status: String,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub selected_index: usize,
    pub message: String,
    pub is_compiling: bool,
    pub moonraker_port: u16,
    pub listen_port: u16,
    pub server_name: String,
    pub max_body_size: String,
    pub fluidd_path: String,
    pub config_prompt_mode: Option<ConfigField>,
    pub input_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    MoonrakerPort,
    ListenPort,
    ServerName,
    MaxBodySize,
    FluiddPath,
}

impl App {
    pub fn new() -> Self {
        Self {
            r_klipp_status: ServiceStatus {
                name: "r_klipp".to_string(),
                active_state: "unknown".to_string(),
                sub_state: "unknown".to_string(),
                load_state: "unknown".to_string(),
                description: "r_klipp service".to_string(),
                main_pid: None,
            },
            rusted_moonraker_status: ServiceStatus {
                name: "rusted_moonraker".to_string(),
                active_state: "unknown".to_string(),
                sub_state: "unknown".to_string(),
                load_state: "unknown".to_string(),
                description: "rusted_moonraker service".to_string(),
                main_pid: None,
            },
            r_klipper_screen_status: ServiceStatus {
                name: "rKlipperScreen".to_string(),
                active_state: "unknown".to_string(),
                sub_state: "unknown".to_string(),
                load_state: "unknown".to_string(),
                description: "rKlipperScreen service".to_string(),
                main_pid: None,
            },
            fluidd_status: "not-found".to_string(),
            mainsail_status: "not-found".to_string(),
            logs: Arc::new(Mutex::new(vec![
                "Welcome to rkiauh (The Rust-based System Provisioner)!".to_string(),
                "Press 'i' to install or 'u' to update the selected component.".to_string(),
                "Press 's', 't', or 'r' to control systemd services (start/stop/restart).".to_string(),
                "Press 'c' to compile/generate Nginx configurations.".to_string(),
            ])),
            selected_index: 0,
            message: "Ready".to_string(),
            is_compiling: false,
            moonraker_port: 7125,
            listen_port: 80,
            server_name: "_".to_string(),
            max_body_size: "50M".to_string(),
            fluidd_path: "/home/jrad/gcode_files".to_string(),
            config_prompt_mode: None,
            input_value: String::new(),
        }
    }

    pub fn selected_component_name(&self) -> &str {
        match self.selected_index {
            0 => "r_klipp",
            1 => "rusted_moonraker",
            2 => "rKlipperScreen",
            3 => "fluidd",
            _ => "mainsail",
        }
    }
}

pub fn draw_dashboard(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Table (Expanded for 5 components)
            Constraint::Min(6),     // Logs / Progress
            Constraint::Length(3),  // Footer / Command Guide
        ])
        .split(f.area());

    // 1. Render Header
    let header_text = vec![
        Line::from(vec![
            Span::styled(" rkiauh ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" | Platform: "),
            Span::styled("MKS SKIPR (RK3328 Cortex-A53)", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" | OS: "),
            Span::styled("Armbian Linux", Style::default().fg(Color::Green)),
            Span::raw(" | Status: "),
            if app.is_compiling {
                Span::styled("COMPILING...", Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK))
            } else {
                Span::styled("IDLE", Style::default().fg(Color::Green))
            }
        ])
    ];
    let header_paragraph = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("System Info"));
    f.render_widget(header_paragraph, chunks[0]);

    // 2. Render Main Service State Table
    let selected_style = Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD);
    
    // Status colors
    let get_status_span = |state: &str, sub: &str| {
        match state {
            "active" => Span::styled("● ACTIVE (running)", Style::default().fg(Color::Green)),
            "failed" => Span::styled("✖ FAILED", Style::default().fg(Color::Red)),
            "inactive" => Span::styled("○ INACTIVE (dead)", Style::default().fg(Color::Gray)),
            _ => Span::styled(format!("? {} ({})", state, sub), Style::default().fg(Color::Yellow)),
        }
    };

    let r_klipp_pid_str = app.r_klipp_status.main_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
    let moonraker_pid_str = app.rusted_moonraker_status.main_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
    let r_klipper_screen_pid_str = app.r_klipper_screen_status.main_pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());

    let fluidd_status_span = match app.fluidd_status.as_str() {
        "active" | "configured" => Span::styled("● CONFIGURED", Style::default().fg(Color::Green)),
        "not-found" => Span::styled("○ NOT CONFIGURED", Style::default().fg(Color::Gray)),
        other => Span::styled(other.to_string(), Style::default().fg(Color::Yellow)),
    };

    let mainsail_status_span = match app.mainsail_status.as_str() {
        "active" | "configured" => Span::styled("● CONFIGURED", Style::default().fg(Color::Green)),
        "not-found" => Span::styled("○ NOT CONFIGURED", Style::default().fg(Color::Gray)),
        other => Span::styled(other.to_string(), Style::default().fg(Color::Yellow)),
    };

    let rows = vec![
        Row::new(vec![
            Cell::from("r_klipp"),
            Cell::from(get_status_span(&app.r_klipp_status.active_state, &app.r_klipp_status.sub_state)),
            Cell::from(r_klipp_pid_str),
            Cell::from("/home/jrad/RustroverProjects/r_klipp-workspace/r_klipp"),
        ]).style(if app.selected_index == 0 { selected_style } else { Style::default() }),

        Row::new(vec![
            Cell::from("rusted_moonraker"),
            Cell::from(get_status_span(&app.rusted_moonraker_status.active_state, &app.rusted_moonraker_status.sub_state)),
            Cell::from(moonraker_pid_str),
            Cell::from("/home/jrad/RustroverProjects/r_klipp-workspace/rusted_moonraker"),
        ]).style(if app.selected_index == 1 { selected_style } else { Style::default() }),

        Row::new(vec![
            Cell::from("rKlipperScreen"),
            Cell::from(get_status_span(&app.r_klipper_screen_status.active_state, &app.r_klipper_screen_status.sub_state)),
            Cell::from(r_klipper_screen_pid_str),
            Cell::from("/home/jrad/RustroverProjects/r_klipp-workspace/rKlipperScreen"),
        ]).style(if app.selected_index == 2 { selected_style } else { Style::default() }),

        Row::new(vec![
            Cell::from("fluidd"),
            Cell::from(fluidd_status_span),
            Cell::from("-"),
            Cell::from("/home/jrad/RustroverProjects/r_klipp-workspace/kiauh/docs"),
        ]).style(if app.selected_index == 3 { selected_style } else { Style::default() }),

        Row::new(vec![
            Cell::from("mainsail"),
            Cell::from(mainsail_status_span),
            Cell::from("-"),
            Cell::from("/home/jrad/RustroverProjects/r_klipp-workspace/mainsail"),
        ]).style(if app.selected_index == 4 { selected_style } else { Style::default() }),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(10),
            Constraint::Percentage(45),
        ]
    )
    .header(
        Row::new(vec!["Component", "Service Status", "PID", "Repository / Config Path"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    )
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Service Monitoring & Control"));
    f.render_widget(table, chunks[1]);

    // 3. Render Compilation / Operation Logs
    let logs_guard = app.logs.lock().unwrap();
    // Get last N lines that fit inside logs block height
    let logs_height = (chunks[2].height as usize).saturating_sub(2);
    let start_idx = logs_guard.len().saturating_sub(logs_height);
    let visible_logs: Vec<Line> = logs_guard[start_idx..]
        .iter()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Gray))))
        .collect();

    let logs_paragraph = Paragraph::new(visible_logs)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Compilation & Installation Live Console"));
    f.render_widget(logs_paragraph, chunks[2]);

    // 4. Render Footer / Command Guide
    let footer_text = Line::from(vec![
        Span::styled(" [↑/↓] Select ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::raw(" | "),
        Span::styled(" [I] Install ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::raw(" | "),
        Span::styled(" [U] Update ", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled(" [S] Start ", Style::default().fg(Color::White).bg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(" [T] Stop ", Style::default().fg(Color::White).bg(Color::Red)),
        Span::raw(" | "),
        Span::styled(" [R] Restart ", Style::default().fg(Color::Black).bg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled(" [C] Configure Nginx ", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" | Msg: "),
        Span::styled(&app.message, Style::default().fg(Color::LightYellow)),
        Span::raw(" | [Q] Quit")
    ]);
    let footer_paragraph = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
    f.render_widget(footer_paragraph, chunks[3]);

    // 5. Configuration Modal Prompt (if active)
    if let Some(field) = app.config_prompt_mode {
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area); // clear background under modal
        
        let prompt_title = match field {
            ConfigField::MoonrakerPort => "Configure Moonraker Port",
            ConfigField::ListenPort => "Configure Nginx Listen Port",
            ConfigField::ServerName => "Configure Server Name (e.g. fluidd.local)",
            ConfigField::MaxBodySize => "Configure Client Max Body Size (e.g. 50M)",
            ConfigField::FluiddPath => "Configure Fluidd Web UI Root Directory Path",
        };

        let placeholder = match field {
            ConfigField::MoonrakerPort => "Default: 7125",
            ConfigField::ListenPort => "Default: 80",
            ConfigField::ServerName => "Default: _",
            ConfigField::MaxBodySize => "Default: 50M",
            ConfigField::FluiddPath => "Default: /home/jrad/RustroverProjects/r_klipp-workspace/kiauh/docs",
        };

        let modal_text = vec![
            Line::from(vec![
                Span::styled(format!("Current Value: {}\n", app.input_value), Style::default().fg(Color::Cyan))
            ]),
            Line::from(vec![
                Span::styled(format!("Type value and press [Enter] to submit. [Esc] to cancel. ({})", placeholder), Style::default().fg(Color::Gray))
            ])
        ];

        let modal_block = Paragraph::new(modal_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Yellow))
                .title(prompt_title));
        
        f.render_widget(modal_block, area);
    }
}

// Helper to center the configuration modal on screen
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
