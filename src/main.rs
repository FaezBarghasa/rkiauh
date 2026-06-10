mod installer;
mod tui;
mod utils;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Offline simulation toggle to allow executing the TUI fully on non-Linux machines
    let simulation_mode = args.contains(&"--offline".to_string()) || cfg!(not(target_os = "linux"));

    // Establish DBus communication proxy early. Fails safely into simulated struct if offline.
    let service_controller = installer::systemd::ServiceController::new(simulation_mode).await?;

    // Start Crossterm UI Event and Render loop (Blocking terminal control transfer)
    if let Err(e) = tui::run_render_loop(simulation_mode, service_controller).await {
        eprintln!("Terminal UI execution ended with an error: {}", e);
    }

    Ok(())
}