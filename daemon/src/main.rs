// #![allow(unused)]
pub mod db;
pub mod process;
pub mod state;

use std::{fmt, fs};

use async_std::process::Command;
use daemonize::Daemonize;
use sea_orm::*;
use state::{init_state, state_dir, LOG};
use sysinfo::{ProcessExt, System, SystemExt};
use zbus::{dbus_interface, fdo, ConnectionBuilder};

use crate::db::{get_db, setup_db};

trait DbusAdaptable<T, E> {
    fn into_dbus(self) -> Result<T, fdo::Error>;
}

impl<T, E: fmt::Display> DbusAdaptable<T, E> for Result<T, E> {
    fn into_dbus(self) -> Result<T, fdo::Error> {
        self.map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

struct ProcessManager {
    db: DatabaseConnection,
    sys: System,
}

#[dbus_interface(name = "org.laccaria.Processes")]
impl ProcessManager {
    async fn start(&self, name: String, command: Vec<String>) -> fdo::Result<()> {
        log::info!("Starting process \"{name}\"");
        let mut proc = Command::new(&command[0])
            .args(&command[1..])
            .spawn()
            .into_dbus()?;

        log::info!("Started \"{name}\" with pid: {}", proc.id());

        if process::Model::find_by_name(&name, &self.db).await.is_ok() {
            return Err(fdo::Error::Failed(
                "Process with the same name already exists".to_string(),
            ));
        }

        match (process::ActiveModel {
            id: NotSet,
            pid: Set(proc.id()),
            name: Set(name),
            status: Set(process::Status::Active),
            command: Set(shlex::join(command.iter().map(String::as_str))),
        }
        .insert(&self.db)
        .await)
        {
            Ok(process_model) => {
                log::info!("Saved process info");

                let conn = self.db.clone();

                async_std::task::spawn(async move {
                    let _status = proc
                        .status()
                        .await
                        .expect("Couldn't obtain exit code of process");

                    let mut process_model = process_model.into_active_model();
                    process_model.status = Set(process::Status::Dead);
                    process_model
                        .update(&conn)
                        .await
                        .expect("Failed to update process entry in database");
                });

                Ok(())
            }
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => Err(fdo::Error::Failed(
                    "Process with the same name already exists".to_string(),
                )),
                _ => unreachable!(),
            },
        }
    }

    async fn kill(&mut self, name: String) -> fdo::Result<()> {
        log::info!("Killing process {name}");
        let proc = process::Model::find_by_name(&name, &self.db).await?;

        if let process::Status::Dead = proc.status {
            return Err(fdo::Error::Failed("Can't kill dead process".into()));
        }

        self.sys.refresh_process((proc.pid as usize).into());
        self.sys
            .process((proc.pid as usize).into())
            .ok_or(fdo::Error::Failed("Couldn't find process".into()))?
            .kill();

        log::info!("Process \"{name}\" succesfully killed");

        let mut proc = proc.into_active_model();
        proc.status = Set(process::Status::Dead);

        Ok(())
    }

    async fn list(&mut self) -> fdo::Result<Vec<(u32, String, u32, f32, f32, bool)>> {
        let procs = process::Entity::find().all(&self.db).await.into_dbus()?;

        Ok(procs
            .into_iter()
            .map(|p| match p.status {
                process::Status::Active => {
                    self.sys.refresh_process((p.pid as usize).into());
                    let sys_proc = self.sys.process((p.pid as usize).into()).unwrap();

                    (
                        p.pid,
                        p.name,
                        (sys_proc.memory() / 1048576) as u32,
                        sys_proc.cpu_usage(),
                        (sys_proc.run_time() as f32 / 60.),
                        p.status as u32 == 1,
                    )
                }
                process::Status::Dead => (p.pid, p.name, 0, 0., 0., p.status as u32 == 1),
            })
            .collect())
    }

    async fn delete(&self, name: &str) -> fdo::Result<()> {
        let proc = process::Model::find_by_name(name, &self.db).await?;

        if let process::Status::Active = proc.status {
            return Err(fdo::Error::Failed("Cannot delete active process".into()));
        }

        proc.delete(&self.db).await.into_dbus()?;

        Ok(())
    }

    async fn restart(&mut self, name: &str, force: bool) -> fdo::Result<()> {
        let process_model = process::Model::find_by_name(name, &self.db).await?;
        if let process::Status::Active = process_model.status {
            if force {
                let pid = (process_model.pid as usize).into();
                self.sys.refresh_process(pid);
                self.sys.process(pid).unwrap().kill();
            } else {
                return Err(fdo::Error::Failed(
                    "Cannot restart running process".to_string(),
                ));
            }
        }
        let command = shlex::split(&process_model.command).unwrap();

        let mut proc = Command::new(&command[0])
            .args(&command[1..])
            .spawn()
            .into_dbus()?;

        let mut process_model = process_model.into_active_model();
        process_model.pid = Set(proc.id());
        process_model.status = Set(process::Status::Active);
        let process_model = process_model.update(&self.db).await.into_dbus()?;

        let conn = self.db.clone();
        async_std::task::spawn(async move {
            let _status = proc
                .status()
                .await
                .expect("Couldn't obtain exit code of process");

            let mut process_model = process_model.into_active_model();
            process_model.status = Set(process::Status::Dead);
            process_model
                .update(&conn)
                .await
                .expect("Failed to update process entry in database");
        });

        Ok(())
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
