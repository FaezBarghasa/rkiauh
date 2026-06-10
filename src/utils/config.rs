use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Basic file parser tailored directly for KIAUH INI styles (kiauh.cfg config parser).
pub struct KiauhConfig {
    pub settings: HashMap<String, String>,
}

impl KiauhConfig {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => String::new(),
        };

        let mut settings = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                settings.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Ok(Self { settings })
    }
}