use anyhow::Result;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

mod installer;
mod tui;
mod utils;

use crate::installer::systemd::{get_service_status, restart_service, start_service, stop_service};
use crate::installer::{
    ComponentInstaller, FluiddInstaller, MainsailInstaller, RKlippInstaller,
    RKlipperScreenInstaller, RustedMoonrakerInstaller,
};
use crate::tui::dashboard::{draw_dashboard, App, ConfigField};
use crate::utils::config::{generate_nginx_config, write_nginx_config, NginxConfigPayload};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup alternate screen and Crossterm raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize App and D-Bus System Connection
    let app = Arc::new(Mutex::new(App::new()));

    let dbus_conn = match zbus::Connection::system().await {
        Ok(c) => Some(c),
        Err(e) => {
            let app_lock = app.lock().unwrap();
            app_lock.logs.lock().unwrap().push(format!(
                "WARNING: Could not connect to system D-Bus: {}. D-Bus actions will fail.",
                e
            ));
            None
        }
    };

    // 3. Spawn background service monitor loop
    let app_clone = app.clone();
    let dbus_clone = dbus_conn.clone();
    tokio::spawn(async move {
        loop {
            if let Some(ref conn) = dbus_clone {
                let r_klipp_res = get_service_status(conn, "r_klipp.service").await;
                let moonraker_res = get_service_status(conn, "rusted_moonraker.service").await;
                let screen_res = get_service_status(conn, "rKlipperScreen.service").await;

                let mut app_lock = app_clone.lock().unwrap();
                if let Ok(st) = r_klipp_res {
                    app_lock.r_klipp_status = st;
                }
                if let Ok(st) = moonraker_res {
                    app_lock.rusted_moonraker_status = st;
                }
                if let Ok(st) = screen_res {
                    app_lock.r_klipper_screen_status = st;
                }
            }
            sleep(Duration::from_millis(1500)).await;
        }
    });

    // Check Nginx configurations on start
    {
        let mut app_lock = app.lock().unwrap();
        if std::path::Path::new("/etc/nginx/sites-available/fluidd").exists() {
            app_lock.fluidd_status = "configured".to_string();
        } else {
            app_lock.fluidd_status = "not-found".to_string();
        }
        if std::path::Path::new("/etc/nginx/sites-available/mainsail").exists() {
            app_lock.mainsail_status = "configured".to_string();
        } else {
            app_lock.mainsail_status = "not-found".to_string();
        }
    }

    // 4. Initialize installers using user-specified repository URLs
    let r_klipp_inst = Arc::new(RKlippInstaller {
        repo_url: "https://github.com/FaezBarghasa/r_klipp".to_string(),
        local_path: "/home/jrad/RustroverProjects/r_klipp-workspace/r_klipp".to_string(),
    });

    let moonraker_inst = Arc::new(RustedMoonrakerInstaller {
        repo_url: "https://github.com/FaezBarghasa/rusted_moonraker".to_string(),
        local_path: "/home/jrad/RustroverProjects/r_klipp-workspace/rusted_moonraker".to_string(),
    });

    let screen_inst = Arc::new(RKlipperScreenInstaller {
        repo_url: "https://github.com/FaezBarghasa/rKlipperScreen".to_string(),
        local_path: "/home/jrad/RustroverProjects/r_klipp-workspace/rKlipperScreen".to_string(),
    });

    let fluidd_inst = Arc::new(FluiddInstaller {
        repo_url: "https://github.com/fluidd-core/fluidd".to_string(),
        local_path: "/home/jrad/RustroverProjects/r_klipp-workspace/kiauh/docs".to_string(),
    });

    let mainsail_inst = Arc::new(MainsailInstaller {
        repo_url: "https://github.com/mainsail-crew/mainsail".to_string(),
        local_path: "/home/jrad/RustroverProjects/r_klipp-workspace/mainsail".to_string(),
    });

    // 5. Main event loop
    let mut reader = crossterm::event::EventStream::new();

    loop {
        // Draw TUI frame
        terminal.draw(|f| {
            let app_lock = app.lock().unwrap();
            draw_dashboard(f, &app_lock);
        })?;

        // Read event asynchronously with safety timeouts
        tokio::select! {
            maybe_event = tokio_stream::StreamExt::next(&mut reader) => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == crossterm::event::KeyEventKind::Release {
                        continue;
                    }

                    let mut app_lock = app.lock().unwrap();

                    // A. Check if configuration wizard is active
                    if let Some(current_field) = app_lock.config_prompt_mode {
                        match key.code {
                            KeyCode::Esc => {
                                app_lock.config_prompt_mode = None;
                                app_lock.message = "Configuration wizard cancelled.".to_string();
                            }
                            KeyCode::Enter => {
                                let val = app_lock.input_value.trim().to_string();
                                match current_field {
                                    ConfigField::MoonrakerPort => {
                                        if let Ok(p) = val.parse::<u16>() {
                                            app_lock.moonraker_port = p;
                                        }
                                        app_lock.input_value = app_lock.listen_port.to_string();
                                        app_lock.config_prompt_mode = Some(ConfigField::ListenPort);
                                    }
                                    ConfigField::ListenPort => {
                                        if let Ok(p) = val.parse::<u16>() {
                                            app_lock.listen_port = p;
                                        }
                                        app_lock.input_value = app_lock.server_name.clone();
                                        app_lock.config_prompt_mode = Some(ConfigField::ServerName);
                                    }
                                    ConfigField::ServerName => {
                                        if !val.is_empty() {
                                            app_lock.server_name = val;
                                        }
                                        app_lock.input_value = app_lock.max_body_size.clone();
                                        app_lock.config_prompt_mode = Some(ConfigField::MaxBodySize);
                                    }
                                    ConfigField::MaxBodySize => {
                                        if !val.is_empty() {
                                            app_lock.max_body_size = val;
                                        }
                                        app_lock.input_value = app_lock.fluidd_path.clone();
                                        app_lock.config_prompt_mode = Some(ConfigField::FluiddPath);
                                    }
                                    ConfigField::FluiddPath => {
                                        if !val.is_empty() {
                                            app_lock.fluidd_path = val;
                                        }
                                        app_lock.config_prompt_mode = None;

                                        // Finalize configuration generation
                                        let payload = NginxConfigPayload {
                                            moonraker_port: app_lock.moonraker_port,
                                            listen_port: app_lock.listen_port,
                                            server_name: app_lock.server_name.clone(),
                                            max_body_size: app_lock.max_body_size.clone(),
                                            fluidd_path: app_lock.fluidd_path.clone(),
                                        };

                                        match generate_nginx_config(&payload) {
                                            Ok(conf) => {
                                                match write_nginx_config(&conf, "/etc/nginx/sites-available/fluidd") {
                                                    Ok(written_path) => {
                                                        app_lock.fluidd_status = "configured".to_string();
                                                        app_lock.message = format!("Nginx conf generated successfully at {}!", written_path);
                                                        app_lock.logs.lock().unwrap().push(format!("Config file saved to: {}", written_path));
                                                    }
                                                    Err(err) => {
                                                        app_lock.message = "Failed to write configuration. Check logs.".to_string();
                                                        app_lock.logs.lock().unwrap().push(err.to_string());
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                app_lock.message = "Template parsing error. Check logs.".to_string();
                                                app_lock.logs.lock().unwrap().push(e.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                app_lock.input_value.push(c);
                            }
                            KeyCode::Backspace => {
                                app_lock.input_value.pop();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // B. Normal keyboard guides
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) || key.code == KeyCode::Char('q') {
                                break;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app_lock.selected_index = (app_lock.selected_index + 1) % 5;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app_lock.selected_index = (app_lock.selected_index + 4) % 5;
                        }
                        KeyCode::Char('s') => {
                            // Start service
                            let name = app_lock.selected_component_name().to_string();
                            if name == "fluidd" || name == "mainsail" {
                                app_lock.message = format!("{} does not use a direct systemd daemon.", name);
                                continue;
                            }
                            let service_name = format!("{}.service", name);
                            if let Some(ref conn) = dbus_conn {
                                let conn_clone = conn.clone();
                                let logs_clone = app_lock.logs.clone();
                                app_lock.message = format!("Starting service {}...", service_name);
                                tokio::spawn(async move {
                                    match start_service(&conn_clone, &service_name).await {
                                        Ok(_) => logs_clone.lock().unwrap().push(format!("Service {} started successfully.", service_name)),
                                        Err(e) => logs_clone.lock().unwrap().push(format!("Error starting {}: {}", service_name, e)),
                                    }
                                });
                            } else {
                                app_lock.message = "No D-Bus connection available.".to_string();
                            }
                        }
                        KeyCode::Char('t') => {
                            // Stop service
                            let name = app_lock.selected_component_name().to_string();
                            if name == "fluidd" || name == "mainsail" {
                                app_lock.message = format!("{} does not use a direct systemd daemon.", name);
                                continue;
                            }
                            let service_name = format!("{}.service", name);
                            if let Some(ref conn) = dbus_conn {
                                let conn_clone = conn.clone();
                                let logs_clone = app_lock.logs.clone();
                                app_lock.message = format!("Stopping service {}...", service_name);
                                tokio::spawn(async move {
                                    match stop_service(&conn_clone, &service_name).await {
                                        Ok(_) => logs_clone.lock().unwrap().push(format!("Service {} stopped.", service_name)),
                                        Err(e) => logs_clone.lock().unwrap().push(format!("Error stopping {}: {}", service_name, e)),
                                    }
                                });
                            } else {
                                app_lock.message = "No D-Bus connection available.".to_string();
                            }
                        }
                        KeyCode::Char('r') => {
                            // Restart service
                            let name = app_lock.selected_component_name().to_string();
                            if name == "fluidd" || name == "mainsail" {
                                app_lock.message = format!("{} does not use a direct systemd daemon.", name);
                                continue;
                            }
                            let service_name = format!("{}.service", name);
                            if let Some(ref conn) = dbus_conn {
                                let conn_clone = conn.clone();
                                let logs_clone = app_lock.logs.clone();
                                app_lock.message = format!("Restarting service {}...", service_name);
                                tokio::spawn(async move {
                                    match restart_service(&conn_clone, &service_name).await {
                                        Ok(_) => logs_clone.lock().unwrap().push(format!("Service {} restarted successfully.", service_name)),
                                        Err(e) => logs_clone.lock().unwrap().push(format!("Error restarting {}: {}", service_name, e)),
                                    }
                                });
                            } else {
                                app_lock.message = "No D-Bus connection available.".to_string();
                            }
                        }
                        KeyCode::Char('c') => {
                            app_lock.input_value = app_lock.moonraker_port.to_string();
                            app_lock.config_prompt_mode = Some(ConfigField::MoonrakerPort);
                            app_lock.message = "Nginx configuration wizard started.".to_string();
                        }
                        KeyCode::Char('i') => {
                            // Install Component
                            if app_lock.is_compiling {
                                app_lock.message = "Another compilation task is already running!".to_string();
                                continue;
                            }
                            app_lock.is_compiling = true;
                            app_lock.message = format!("Installing {}...", app_lock.selected_component_name());

                            let component_idx = app_lock.selected_index;
                            let logs_clone = app_lock.logs.clone();
                            let app_ref = app.clone();
                            let r_klipp_clone = r_klipp_inst.clone();
                            let moonraker_clone = moonraker_inst.clone();
                            let screen_clone = screen_inst.clone();
                            let fluidd_clone = fluidd_inst.clone();
                            let mainsail_clone = mainsail_inst.clone();

                            tokio::spawn(async move {
                                let res = match component_idx {
                                    0 => {
                                        let clone_res = r_klipp_clone.clone_repo(logs_clone.clone());
                                        if clone_res.is_ok() {
                                            let comp_res = r_klipp_clone.compile(logs_clone.clone());
                                            if comp_res.is_ok() {
                                                r_klipp_clone.install_service(logs_clone.clone())
                                            } else {
                                                comp_res
                                            }
                                        } else {
                                            clone_res
                                        }
                                    }
                                    1 => {
                                        let clone_res = moonraker_clone.clone_repo(logs_clone.clone());
                                        if clone_res.is_ok() {
                                            let comp_res = moonraker_clone.compile(logs_clone.clone());
                                            if comp_res.is_ok() {
                                                moonraker_clone.install_service(logs_clone.clone())
                                            } else {
                                                comp_res
                                            }
                                        } else {
                                            clone_res
                                        }
                                    }
                                    2 => {
                                        let clone_res = screen_clone.clone_repo(logs_clone.clone());
                                        if clone_res.is_ok() {
                                            let comp_res = screen_clone.compile(logs_clone.clone());
                                            if comp_res.is_ok() {
                                                screen_clone.install_service(logs_clone.clone())
                                            } else {
                                                comp_res
                                            }
                                        } else {
                                            clone_res
                                        }
                                    }
                                    3 => {
                                        let clone_res = fluidd_clone.clone_repo(logs_clone.clone());
                                        if clone_res.is_ok() {
                                            fluidd_clone.compile(logs_clone.clone())
                                        } else {
                                            clone_res
                                        }
                                    }
                                    _ => {
                                        let clone_res = mainsail_clone.clone_repo(logs_clone.clone());
                                        if clone_res.is_ok() {
                                            mainsail_clone.compile(logs_clone.clone())
                                        } else {
                                            clone_res
                                        }
                                    }
                                };

                                let mut final_app = app_ref.lock().unwrap();
                                final_app.is_compiling = false;
                                match res {
                                    Ok(_) => final_app.message = "Installation completed successfully!".to_string(),
                                    Err(_) => final_app.message = "Installation failed. Check compiler log.".to_string(),
                                }
                            });
                        }
                        KeyCode::Char('u') => {
                            // Update Component
                            if app_lock.is_compiling {
                                app_lock.message = "Another compilation task is already running!".to_string();
                                continue;
                            }
                            app_lock.is_compiling = true;
                            app_lock.message = format!("Updating {}...", app_lock.selected_component_name());

                            let component_idx = app_lock.selected_index;
                            let logs_clone = app_lock.logs.clone();
                            let app_ref = app.clone();
                            let r_klipp_clone = r_klipp_inst.clone();
                            let moonraker_clone = moonraker_inst.clone();
                            let screen_clone = screen_inst.clone();
                            let fluidd_clone = fluidd_inst.clone();
                            let mainsail_clone = mainsail_inst.clone();

                            tokio::spawn(async move {
                                let res = match component_idx {
                                    0 => {
                                        let pull_res = r_klipp_clone.pull_repo(logs_clone.clone());
                                        if pull_res.is_ok() {
                                            r_klipp_clone.compile(logs_clone.clone())
                                        } else {
                                            pull_res
                                        }
                                    }
                                    1 => {
                                        let pull_res = moonraker_clone.pull_repo(logs_clone.clone());
                                        if pull_res.is_ok() {
                                            moonraker_clone.compile(logs_clone.clone())
                                        } else {
                                            pull_res
                                        }
                                    }
                                    2 => {
                                        let pull_res = screen_clone.pull_repo(logs_clone.clone());
                                        if pull_res.is_ok() {
                                            screen_clone.compile(logs_clone.clone())
                                        } else {
                                            pull_res
                                        }
                                    }
                                    3 => {
                                        let pull_res = fluidd_clone.pull_repo(logs_clone.clone());
                                        if pull_res.is_ok() {
                                            fluidd_clone.compile(logs_clone.clone())
                                        } else {
                                            pull_res
                                        }
                                    }
                                    _ => {
                                        let pull_res = mainsail_clone.pull_repo(logs_clone.clone());
                                        if pull_res.is_ok() {
                                            mainsail_clone.compile(logs_clone.clone())
                                        } else {
                                            pull_res
                                        }
                                    }
                                };

                                let mut final_app = app_ref.lock().unwrap();
                                final_app.is_compiling = false;
                                match res {
                                    Ok(_) => final_app.message = "Update completed successfully!".to_string(),
                                    Err(_) => final_app.message = "Update failed. Check compiler log.".to_string(),
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ = sleep(Duration::from_millis(50)) => {}
        }
    }

    // 6. Restore terminal alternate screen and disable raw mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
