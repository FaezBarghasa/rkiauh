use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct SysConfigurator {
    pub simulation_mode: bool,
}

impl SysConfigurator {
    pub fn new(simulation_mode: bool) -> Self {
        Self { simulation_mode }
    }

    pub fn write_udev_rule(&self, dest_path: &Path, content: &str) -> io::Result<()> {
        if self.simulation_mode {
            return Ok(());
        }
        if dest_path.exists() {
            let backup_path = dest_path.with_extension("bak");
            fs::copy(dest_path, backup_path)?;
        }
        let mut file = fs::File::create(dest_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn write_dt_overlays(&self, env_path: &Path, new_overlays: &[&str]) -> io::Result<()> {
        if self.simulation_mode {
            return Ok(());
        }
        if !env_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "armbianEnv.txt config space unavailable.",
            ));
        }
        
        let content = fs::read_to_string(env_path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut flag_mutated = false;

        for line in lines.iter_mut() {
            if line.starts_with("overlays=") {
                flag_mutated = true;
                let current_overlays: Vec<&str> = line["overlays=".len()..].split_whitespace().collect();
                let updated_nodes: Vec<&str> = [current_overlays, new_overlays.to_vec()].concat();
                let mut unique_nodes: Vec<&str> = vec![];
                for node in updated_nodes {
                    if !unique_nodes.contains(&node) { unique_nodes.push(node); }
                }
                *line = format!("overlays={}", unique_nodes.join(" "));
                break;
            }
        }
        if !flag_mutated {
            lines.push(format!("overlays={}", new_overlays.join(" ")));
        }

        let backup_path = env_path.with_extension("bak");
        fs::copy(env_path, backup_path)?;
        fs::write(env_path, lines.join("\n") + "\n")
    }
}