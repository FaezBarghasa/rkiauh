use anyhow::{Context, Result};
use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait Manager {
    fn get_unit(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn restart_unit(&self, name: &str, mode: &str)
        -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
pub trait Unit {
    #[zbus(property)]
    fn active_state(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn sub_state(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn load_state(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn description(&self) -> zbus::Result<String>;
}

#[proxy(
    interface = "org.freedesktop.systemd1.Service",
    default_service = "org.freedesktop.systemd1"
)]
pub trait Service {
    #[zbus(property)]
    fn main_pid(&self) -> zbus::Result<u32>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub active_state: String,
    pub sub_state: String,
    pub load_state: String,
    pub description: String,
    pub main_pid: Option<u32>,
}

pub async fn get_service_status(conn: &zbus::Connection, name: &str) -> Result<ServiceStatus> {
    let manager = ManagerProxy::new(conn).await?;

    let path = match manager.get_unit(name).await {
        Ok(p) => p,
        Err(e) => {
            return Ok(ServiceStatus {
                name: name.to_string(),
                active_state: "inactive".to_string(),
                sub_state: "dead".to_string(),
                load_state: "not-found".to_string(),
                description: format!("No unit found: {}", e),
                main_pid: None,
            });
        }
    };

    let unit_proxy = UnitProxy::builder(conn).path(path.clone())?.build().await?;

    let active_state = unit_proxy
        .active_state()
        .await
        .unwrap_or_else(|_| "unknown".to_string());
    let sub_state = unit_proxy
        .sub_state()
        .await
        .unwrap_or_else(|_| "unknown".to_string());
    let load_state = unit_proxy
        .load_state()
        .await
        .unwrap_or_else(|_| "unknown".to_string());
    let description = unit_proxy
        .description()
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let mut main_pid = None;
    if active_state == "active" {
        if let Ok(service_proxy) = ServiceProxy::builder(conn).path(path)?.build().await {
            if let Ok(pid) = service_proxy.main_pid().await {
                if pid > 0 {
                    main_pid = Some(pid);
                }
            }
        }
    }

    Ok(ServiceStatus {
        name: name.to_string(),
        active_state,
        sub_state,
        load_state,
        description,
        main_pid,
    })
}

pub async fn start_service(conn: &zbus::Connection, name: &str) -> Result<()> {
    let manager = ManagerProxy::new(conn).await?;
    manager
        .start_unit(name, "replace")
        .await
        .context(format!("Failed to start service {}", name))?;
    Ok(())
}

pub async fn stop_service(conn: &zbus::Connection, name: &str) -> Result<()> {
    let manager = ManagerProxy::new(conn).await?;
    manager
        .stop_unit(name, "replace")
        .await
        .context(format!("Failed to stop service {}", name))?;
    Ok(())
}

pub async fn restart_service(conn: &zbus::Connection, name: &str) -> Result<()> {
    let manager = ManagerProxy::new(conn).await?;
    manager
        .restart_unit(name, "replace")
        .await
        .context(format!("Failed to restart service {}", name))?;
    Ok(())
}
