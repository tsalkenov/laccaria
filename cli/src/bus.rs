use zbus::{dbus_proxy, Result};

#[dbus_proxy(
    interface = "org.laccaria.Processes",
    default_service = "org.laccaria.Processes",
    default_path = "/org/laccaria/Processes"
)]
pub trait ProcessManager {
    async fn start(&self, name: &str, command: Vec<String>) -> Result<()>;
    async fn kill(&self, name: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<(u32, String, u32, f32, u32, bool)>>;
    async fn delete(&self, name: &str) -> Result<()>;
}
