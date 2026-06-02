pub mod systemd;

use std::path::Path;
use std::process::{Stdio, Command};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use anyhow::{Result, Context};
use git2::{Repository, FetchOptions, ResetType, build::CheckoutBuilder};

pub trait ComponentInstaller: Send + Sync {
    fn name(&self) -> &str;
    fn get_repo_url(&self) -> &str;
    fn get_local_path(&self) -> &str;
    fn is_installed(&self) -> bool;
    
    fn clone_repo(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let path = self.get_local_path();
        let url = self.get_repo_url();
        {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Cloning {} into {}...", url, path));
        }

        if Path::new(path).exists() {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Path {} already exists. Skipping clone.", path));
            return Ok(());
        }

        Repository::clone(url, path)?;
        
        let mut buf = log_buf.lock().unwrap();
        buf.push(format!("Successfully cloned {}!", self.name()));
        Ok(())
    }

    fn pull_repo(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let path = self.get_local_path();
        {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Updating repository at {}...", path));
        }

        let repo = Repository::open(path)?;
        let mut remote = repo.find_remote("origin")
            .context("Failed to find remote 'origin'")?;

        let mut fetch_options = FetchOptions::new();
        
        let mut buf = log_buf.lock().unwrap();
        buf.push("Fetching latest updates from remote...".to_string());
        drop(buf);

        remote.fetch(&["master"], Some(&mut fetch_options), None)
            .or_else(|_| remote.fetch(&["main"], Some(&mut fetch_options), None))
            .context("Fetch failed")?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        
        let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;
        let mut buf = log_buf.lock().unwrap();
        
        if analysis.is_fast_forward() {
            buf.push("Fast-forward merge possible. Merging...".to_string());
            let refname = if repo.find_reference("refs/heads/master").is_ok() {
                "refs/heads/master"
            } else {
                "refs/heads/main"
            };
            let mut reference = repo.find_reference(refname)?;
            let msg = format!("Fast-Forward: Setting to {}", fetch_commit.id());
            reference.set_target(fetch_commit.id(), &msg)?;
            repo.set_head(refname)?;
            repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
            buf.push("Update completed via fast-forward.".to_string());
        } else if analysis.is_up_to_date() {
            buf.push("Repository is already up-to-date.".to_string());
        } else {
            buf.push("Non-fast-forward state. Performing hard reset to origin tip...".to_string());
            let target_obj = repo.find_object(fetch_commit.id(), None)?;
            repo.reset(&target_obj, ResetType::Hard, None)?;
            buf.push("Hard reset completed successfully.".to_string());
        }
        
        Ok(())
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()>;
    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()>;
}

pub struct RKlippInstaller {
    pub repo_url: String,
    pub local_path: String,
}

impl ComponentInstaller for RKlippInstaller {
    fn name(&self) -> &str {
        "r_klipp"
    }

    fn get_repo_url(&self) -> &str {
        &self.repo_url
    }

    fn get_local_path(&self) -> &str {
        &self.local_path
    }

    fn is_installed(&self) -> bool {
        Path::new(&self.local_path).exists()
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let manifest_path = format!("{}/Cargo.toml", self.local_path);
        {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Compiling r_klipp using manifest: {}", manifest_path));
        }

        let mut child = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--release")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start cargo compilation")?;

        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);

        for line in reader.lines() {
            if let Ok(line_str) = line {
                let mut buf = log_buf.lock().unwrap();
                buf.push(line_str);
            }
        }

        let status = child.wait()?;
        let mut buf = log_buf.lock().unwrap();
        if status.success() {
            buf.push("r_klipp compilation finished successfully!".to_string());
            Ok(())
        } else {
            buf.push(format!("r_klipp compilation failed with exit status: {}", status));
            Err(anyhow::anyhow!("Cargo build failed"))
        }
    }

    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let service_content = r#"[Unit]
Description=r_klipp Service
After=network.target

[Service]
Type=simple
User=jrad
ExecStart=/home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/target/release/klipper-host
Restart=always

[Install]
WantedBy=multi-user.target
"#;
        
        let service_path = "/etc/systemd/system/r_klipp.service";
        let mut buf = log_buf.lock().unwrap();
        buf.push(format!("Writing systemd service to {}...", service_path));
        
        match std::fs::write(service_path, service_content) {
            Ok(_) => {
                buf.push("Successfully wrote systemd service file.".to_string());
                Ok(())
            }
            Err(e) => {
                buf.push(format!("Failed writing systemd service: {}. Attempting fallback writing locally.", e));
                let local_service = format!("{}/r_klipp.service", self.local_path);
                std::fs::write(&local_service, service_content)?;
                buf.push(format!("Wrote service file locally to {}.", local_service));
                Ok(())
            }
        }
    }
}

pub struct RustedMoonrakerInstaller {
    pub repo_url: String,
    pub local_path: String,
}

impl ComponentInstaller for RustedMoonrakerInstaller {
    fn name(&self) -> &str {
        "rusted_moonraker"
    }

    fn get_repo_url(&self) -> &str {
        &self.repo_url
    }

    fn get_local_path(&self) -> &str {
        &self.local_path
    }

    fn is_installed(&self) -> bool {
        Path::new(&self.local_path).exists()
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let manifest_path = format!("{}/Cargo.toml", self.local_path);
        {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Compiling rusted_moonraker using manifest: {}", manifest_path));
        }

        let mut child = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--release")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start cargo compilation")?;

        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);

        for line in reader.lines() {
            if let Ok(line_str) = line {
                let mut buf = log_buf.lock().unwrap();
                buf.push(line_str);
            }
        }

        let status = child.wait()?;
        let mut buf = log_buf.lock().unwrap();
        if status.success() {
            buf.push("rusted_moonraker compilation finished successfully!".to_string());
            Ok(())
        } else {
            buf.push(format!("rusted_moonraker compilation failed with exit status: {}", status));
            Err(anyhow::anyhow!("Cargo build failed"))
        }
    }

    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let service_content = r#"[Unit]
Description=Rusted Moonraker Service
After=network.target

[Service]
Type=simple
User=jrad
ExecStart=/home/jrad/RustroverProjects/r_klipp-workspace/rusted_moonraker/target/release/rmr-app
Restart=always

[Install]
WantedBy=multi-user.target
"#;
        
        let service_path = "/etc/systemd/system/rusted_moonraker.service";
        let mut buf = log_buf.lock().unwrap();
        buf.push(format!("Writing systemd service to {}...", service_path));
        
        match std::fs::write(service_path, service_content) {
            Ok(_) => {
                buf.push("Successfully wrote systemd service file.".to_string());
                Ok(())
            }
            Err(e) => {
                buf.push(format!("Failed writing systemd service: {}. Attempting fallback writing locally.", e));
                let local_service = format!("{}/rusted_moonraker.service", self.local_path);
                std::fs::write(&local_service, service_content)?;
                buf.push(format!("Wrote service file locally to {}.", local_service));
                Ok(())
            }
        }
    }
}

pub struct FluiddInstaller {
    pub repo_url: String,
    pub local_path: String,
}

impl ComponentInstaller for FluiddInstaller {
    fn name(&self) -> &str {
        "fluidd"
    }

    fn get_repo_url(&self) -> &str {
        &self.repo_url
    }

    fn get_local_path(&self) -> &str {
        &self.local_path
    }

    fn is_installed(&self) -> bool {
        Path::new(&self.local_path).exists()
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let mut buf = log_buf.lock().unwrap();
        buf.push("Fluidd is a precompiled web frontend. No compilation required.".to_string());
        Ok(())
    }

    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let mut buf = log_buf.lock().unwrap();
        buf.push("Fluidd does not have a background daemon service. It runs via Nginx.".to_string());
        Ok(())
    }
}

pub struct RKlipperScreenInstaller {
    pub repo_url: String,
    pub local_path: String,
}

impl ComponentInstaller for RKlipperScreenInstaller {
    fn name(&self) -> &str {
        "rKlipperScreen"
    }

    fn get_repo_url(&self) -> &str {
        &self.repo_url
    }

    fn get_local_path(&self) -> &str {
        &self.local_path
    }

    fn is_installed(&self) -> bool {
        Path::new(&self.local_path).exists()
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let manifest_path = format!("{}/Cargo.toml", self.local_path);
        {
            let mut buf = log_buf.lock().unwrap();
            buf.push(format!("Compiling rKlipperScreen using manifest: {}", manifest_path));
        }

        let mut child = Command::new("cargo")
            .arg("build")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--release")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start cargo compilation")?;

        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);

        for line in reader.lines() {
            if let Ok(line_str) = line {
                let mut buf = log_buf.lock().unwrap();
                buf.push(line_str);
            }
        }

        let status = child.wait()?;
        let mut buf = log_buf.lock().unwrap();
        if status.success() {
            buf.push("rKlipperScreen compilation finished successfully!".to_string());
            Ok(())
        } else {
            buf.push(format!("rKlipperScreen compilation failed with exit status: {}", status));
            Err(anyhow::anyhow!("Cargo build failed"))
        }
    }

    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let service_content = r#"[Unit]
Description=rKlipperScreen Service
After=network.target

[Service]
Type=simple
User=jrad
ExecStart=/home/jrad/RustroverProjects/r_klipp-workspace/rKlipperScreen/target/release/rKlipperScreen
Restart=always

[Install]
WantedBy=multi-user.target
"#;
        
        let service_path = "/etc/systemd/system/rKlipperScreen.service";
        let mut buf = log_buf.lock().unwrap();
        buf.push(format!("Writing systemd service to {}...", service_path));
        
        match std::fs::write(service_path, service_content) {
            Ok(_) => {
                buf.push("Successfully wrote systemd service file.".to_string());
                Ok(())
            }
            Err(e) => {
                buf.push(format!("Failed writing systemd service: {}. Attempting fallback writing locally.", e));
                let local_service = format!("{}/rKlipperScreen.service", self.local_path);
                std::fs::write(&local_service, service_content)?;
                buf.push(format!("Wrote service file locally to {}.", local_service));
                Ok(())
            }
        }
    }
}

pub struct MainsailInstaller {
    pub repo_url: String,
    pub local_path: String,
}

impl ComponentInstaller for MainsailInstaller {
    fn name(&self) -> &str {
        "mainsail"
    }

    fn get_repo_url(&self) -> &str {
        &self.repo_url
    }

    fn get_local_path(&self) -> &str {
        &self.local_path
    }

    fn is_installed(&self) -> bool {
        Path::new(&self.local_path).exists()
    }

    fn compile(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let mut buf = log_buf.lock().unwrap();
        buf.push("Mainsail is a precompiled web frontend. No compilation required.".to_string());
        Ok(())
    }

    fn install_service(&self, log_buf: Arc<Mutex<Vec<String>>>) -> Result<()> {
        let mut buf = log_buf.lock().unwrap();
        buf.push("Mainsail does not have a background daemon service. It runs via Nginx.".to_string());
        Ok(())
    }
}
