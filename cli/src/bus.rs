use zbus::{dbus_proxy, Result};

#[dbus_proxy(
    interface = "org.amanita.ProcessManager",
    default_service = "org.amanita.ProcessManager",
    default_path = "/org/amanita/ProcessManager"
)]
pub trait ProcessManager {
    async fn start(&self, name: &str, command: Vec<String>) -> Result<()>;
    async fn kill(&self, name: &str) -> Result<()>;
    async fn list(&self) -> Result<()>;
    async fn delete(&self, name: &str) -> Result<()>;
}
