use std::path::PathBuf;
use crate::utils::sys::SysConfigurator;

pub trait Command {
    async fn execute(&mut self) -> Result<(), String>;
    async fn rollback(&mut self) -> Result<(), String>;
}

pub struct InstallUdevRuleCommand {
    pub path: PathBuf,
    pub content: String,
    pub configurer: SysConfigurator,
    executed: bool,
}

impl InstallUdevRuleCommand {
    pub fn new(path: PathBuf, content: String, simulation_mode: bool) -> Self {
        Self {
            path,
            content,
            configurer: SysConfigurator::new(simulation_mode),
            executed: false,
        }
    }
}

impl Command for InstallUdevRuleCommand {
    async fn execute(&mut self) -> Result<(), String> {
        self.configurer
            .write_udev_rule(&self.path, &self.content)
            .map_err(|e| format!("Failed to configure udev rule safely: {}", e))?;
        self.executed = true;
        Ok(())
    }

    async fn rollback(&mut self) -> Result<(), String> {
        if self.executed {
            let backup = self.path.with_extension("bak");
            if backup.exists() && !self.configurer.simulation_mode {
                std::fs::copy(&backup, &self.path).map_err(|e| {
                    format!("Rollback failed, system may be in unexpected state: {}", e)
                })?;
                std::fs::remove_file(&backup)
                    .map_err(|e| format!("Could not purge backup artifact: {}", e))?;
            } else if !backup.exists() && !self.configurer.simulation_mode {
                std::fs::remove_file(&self.path).unwrap_or(());
            }
            self.executed = false;
        }
        Ok(())
    }
}