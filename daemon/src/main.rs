// #![allow(unused)]
pub mod db;
pub mod process;
pub mod state;

use std::fs;

use async_std::process::Command;
use daemonize::Daemonize;
use sea_orm::*;
use state::{init_state, state_dir, LOG};
use sysinfo::{System, SystemExt};
use zbus::{dbus_interface, ConnectionBuilder};

use crate::db::{get_db, setup_db};

struct ProcessManager {
    db: DatabaseConnection,
    sys: System,
}

#[dbus_interface(name = "org.laccaria.Processes")]
impl ProcessManager {
    async fn start(&self, name: String, command: Vec<String>) {
        match Command::new(&command[0]).args(&command[1..]).spawn() {
            Err(e) => log::error!("Failed to start process {name}: {e}"),
            Ok(p) => {
                log::info!("Started {name} with pid: {}", p.id());
                process::ActiveModel {
                    pid: Set(p.id()),
                    name: Set(name),
                }
                .insert(&self.db)
                .await
                .expect("Failed to save process");
            }
        };
    }

    async fn kill(&mut self, name: String) {
        log::info!("Killing process {name}");
        let proc = process::Entity::find()
            .filter(process::Column::Name.eq(name))
            .one(&self.db)
            .await
            .expect("Failed to get process")
            .expect("No process found");

        self.sys
            .process((proc.pid as usize).into())
            .expect("Failed to get process by id");
    }

    async fn list(&self) {}

    async fn delete(&self, _name: &str) {}

    async fn nuke(&self) {
        log::info!("Self destruct initiated");
        std::process::exit(0);
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    init_state()?;

    if let Err(e) = Daemonize::new()
        .stdout(fs::File::create(state_dir().join(LOG))?)
        .stderr(fs::File::create(state_dir().join(LOG))?)
        .start()
    {
        log::error!("Failed to daemonize process manager: {e}")
    }

    async_std::task::block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let db = get_db().await?;
    let mut sys = System::new();
    sys.refresh_processes();

    setup_db(&db).await;
    let process_manager = ProcessManager { db, sys };
    let _connection = ConnectionBuilder::session()?
        .name("org.laccaria.Processes")?
        .serve_at("/org/laccaria/Processes", process_manager)?
        .build()
        .await?;

    log::info!("Process manager started");

    std::future::pending::<anyhow::Result<()>>().await
}
