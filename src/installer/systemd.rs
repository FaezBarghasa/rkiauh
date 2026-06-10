use zbus::{proxy, Connection, Result as ZbusResult};

#[proxy(
    interface = "org.freedesktop.systemd1.Manager",
    default_service = "org.freedesktop.systemd1",
    default_path = "/org/freedesktop/systemd1"
)]
pub trait SystemdManager {
    fn start_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn stop_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn restart_unit(&self, name: &str, mode: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    fn get_unit(&self, name: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

/// Controls direct interaction with the system bus, dropping the requirement for executing
/// subprocess strings (e.g. systemctl)
pub struct ServiceController {
    connection: Option<Connection>,
    simulation_mode: bool,
}

impl ServiceController {
    pub async fn new(simulation_mode: bool) -> ZbusResult<Self> {
        if simulation_mode {
            return Ok(Self {
                connection: None,
                simulation_mode,
            });
        }
        let connection = Connection::system().await?;
        Ok(Self {
            connection: Some(connection),
            simulation_mode,
        })
    }

    pub async fn start_service(&self, name: &str) -> ZbusResult<()> {
        if self.simulation_mode {
            return Ok(());
        }
        if let Some(conn) = &self.connection {
            let proxy = SystemdManagerProxy::new(conn).await?;
            proxy.start_unit(name, "replace").await?;
        }
        Ok(())
    }

    pub async fn stop_service(&self, name: &str) -> ZbusResult<()> {
        if self.simulation_mode {
            return Ok(());
        }
        if let Some(conn) = &self.connection {
            let proxy = SystemdManagerProxy::new(conn).await?;
            proxy.stop_unit(name, "replace").await?;
        }
        Ok(())
    }

    pub async fn restart_service(&self, name: &str) -> ZbusResult<()> {
        if self.simulation_mode {
            return Ok(());
        }
        if let Some(conn) = &self.connection {
            let proxy = SystemdManagerProxy::new(conn).await?;
            proxy.restart_unit(name, "replace").await?;
        }
        Ok(())
    }

    pub async fn verify_service_exists(&self, name: &str) -> ZbusResult<bool> {
        if self.simulation_mode {
            return Ok(true); // Return affirmative stub for testing UI interactions natively
        }
        let proxy = SystemdManagerProxy::new(self.connection.as_ref().unwrap()).await?;
        Ok(proxy.get_unit(name).await.is_ok())
    }
}