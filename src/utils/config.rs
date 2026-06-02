use tera::{Tera, Context};
use serde::Serialize;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context as AnyhowContext};

// Embed default Nginx template
const NGINX_TEMPLATE: &str = r#"
upstream moonraker {
    server 127.0.0.1:{{ moonraker_port }};
}

server {
    listen {{ listen_port }} default_server;
    listen [::]:{{ listen_port }} default_server;

    server_name {{ server_name }};

    client_max_body_size {{ max_body_size }};

    root {{ fluidd_path }};
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api {
        proxy_pass http://moonraker;
        proxy_set_header Host $http_host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Scheme $scheme;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location /websocket {
        proxy_pass http://moonraker;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $http_host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_read_timeout 86400;
    }
}
"#;

#[derive(Serialize)]
pub struct NginxConfigPayload {
    pub moonraker_port: u16,
    pub listen_port: u16,
    pub server_name: String,
    pub max_body_size: String,
    pub fluidd_path: String,
}

pub fn generate_nginx_config(payload: &NginxConfigPayload) -> Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template("nginx.conf", NGINX_TEMPLATE)
        .context("Failed to parse embedded nginx template")?;
    
    let context = Context::from_serialize(payload)
        .context("Failed to serialize nginx payload into Tera context")?;
    
    let rendered = tera.render("nginx.conf", &context)
        .context("Failed to render Nginx template")?;
    
    Ok(rendered)
}

pub fn write_nginx_config(rendered_conf: &str, target_path: &str) -> Result<String> {
    let path = Path::new(target_path);
    
    if let Some(parent) = path.parent() {
        // Attempt to create parent directories, ignoring failure if permissions limit it
        let _ = fs::create_dir_all(parent);
    }

    match fs::write(path, rendered_conf) {
        Ok(_) => Ok(target_path.to_string()),
        Err(e) => {
            // Fallback: write to a local path or user-specific path if root permissions aren't present
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let fallback_dir = format!("{}/.config/rkiauh", home);
            fs::create_dir_all(&fallback_dir)?;
            let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("fluidd.conf");
            let fallback_path = format!("{}/{}", fallback_dir, filename);
            fs::write(&fallback_path, rendered_conf)?;
            Err(anyhow::anyhow!("Permission denied writing to {}: {}. Saved fallback configuration to: {}", target_path, e, fallback_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nginx_config_generation() {
        let payload = NginxConfigPayload {
            moonraker_port: 7125,
            listen_port: 8080,
            server_name: "fluidd.test".to_string(),
            max_body_size: "100M".to_string(),
            fluidd_path: "/var/www/fluidd".to_string(),
        };

        let result = generate_nginx_config(&payload);
        assert!(result.is_ok());
        let config = result.unwrap();

        assert!(config.contains("server 127.0.0.1:7125;"));
        assert!(config.contains("listen 8080 default_server;"));
        assert!(config.contains("server_name fluidd.test;"));
        assert!(config.contains("client_max_body_size 100M;"));
        assert!(config.contains("root /var/www/fluidd;"));
    }
}
