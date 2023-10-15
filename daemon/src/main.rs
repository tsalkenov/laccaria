// #![allow(unused)]
pub mod state;

use std::fs;

use async_std::process::Command;
use daemonize::Daemonize;
use state::state_dir;
use zbus::{dbus_interface, ConnectionBuilder};

use crate::state::{ACCESS_LOG, ERROR_LOG};

struct ProcessManager;

#[dbus_interface(name = "org.laccaria.Processes")]
impl ProcessManager {
    async fn start(&self, name: &str, command: Vec<String>) {
        if let Err(e) = Command::new(&command[0]).args(&command[1..]).spawn() {
            log::error!("Failed to start process {name}: {e}")
        };
    }

    async fn kill(&self, name: &str) {}

    async fn list(&self) {}

    async fn delete(&self, name: &str) {}

    async fn nuke(&self) {
        log::info!("Self destruct initiated");
        std::process::exit(0);
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    if let Err(e) = Daemonize::new()
        .stdout(fs::File::create(state_dir().join(ACCESS_LOG))?)
        .stderr(fs::File::create(state_dir().join(ERROR_LOG))?)
        .start()
    {
        log::error!("Failed to daemonize process manager: {e}")
    }

    async_std::task::block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let process_manager = ProcessManager;

    let _connection = ConnectionBuilder::session()?
        .name("org.laccaria.Processes")?
        .serve_at("/org/laccaria/Processes", process_manager)?
        .build()
        .await?;

    log::info!("Process manager started");

    std::future::pending::<anyhow::Result<()>>().await
}
