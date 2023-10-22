use zbus::{dbus_proxy, Result};

#[dbus_proxy(
    interface = "org.laccaria.Processes",
    default_service = "org.laccaria.Processes",
    default_path = "/org/laccaria/Processes"
)]
pub trait ProcessManager {
    async fn start(&self, name: &str, restart: bool, command: &str) -> Result<()>;
    async fn kill(&self, name: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<(u32, String, u32, f32, f32, bool, bool)>>;
    async fn delete(&self, name: &str) -> Result<()>;
    async fn restart(&self, name: &str, force: bool) -> Result<()>;
}
