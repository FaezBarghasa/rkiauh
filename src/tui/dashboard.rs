use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table},
    Frame,
};
use sysinfo::System;

pub struct AppState {
    pub system: System,
    pub services: Vec<(String, String)>,
    pub simulation_mode: bool,
}

impl AppState {
    pub fn new(simulation_mode: bool) -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            services: vec![
                ("klipper.service".to_string(), "active".to_string()),
                ("moonraker.service".to_string(), "inactive".to_string()),
                ("crowsnest.service".to_string(), "active".to_string()),
            ],
            simulation_mode,
        }
    }

    pub fn update(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
    }
}

pub fn draw(f: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_text = if state.simulation_mode {
        "rkiauh Administration Platform - Offline Target System Verification"
    } else {
        "rkiauh Administration Platform"
    };

    let header = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let rows = state.services.iter().map(|(s, status)| {
        let style = if status == "active" {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };
        Row::new(vec![s.as_str(), status.as_str()]).style(style)
    });

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(Row::new(vec!["Service Tracker", "Status"]).style(Style::default().fg(Color::Yellow)))
    .block(Block::default().title("Zbus Service Bridge").borders(Borders::ALL));
    f.render_widget(table, chunks[1]);

    let cpu_usage = state.system.global_cpu_info().cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU Usage Target").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent((cpu_usage as u16).min(100));
    f.render_widget(cpu_gauge, chunks[2]);

    let mem_used = state.system.used_memory() as f64;
    let mem_total = state.system.total_memory() as f64;
    let mem_percent = if mem_total > 0.0 { ((mem_used / mem_total) * 100.0) as u16 } else { 0 };
    
    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory Allocation").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::LightBlue))
        .percent(mem_percent.min(100));
    f.render_widget(mem_gauge, chunks[3]);
}