// #![allow(unused)]
pub mod active_process;
pub mod db;
pub mod process;
pub mod state;

use std::{fmt, fs};

use bitcode::{decode, encode};
use daemonize::Daemonize;
use db::Db;
use sled::CompareAndSwapError;
use state::{init_state, state_dir, LOG};
use sysinfo::{ProcessExt, System, SystemExt};
use zbus::{dbus_interface, fdo, ConnectionBuilder};

use crate::{
    active_process::ActiveProcess,
    db::get_db,
    process::{Process, Status},
};

trait DbusAdaptable<T, E> {
    fn into_dbus(self) -> Result<T, fdo::Error>;
}

impl<T, E: fmt::Display> DbusAdaptable<T, E> for Result<T, E> {
    fn into_dbus(self) -> Result<T, fdo::Error> {
        self.map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

struct ProcessManager {
    db: Db,
    sys: System,
}

#[dbus_interface(name = "org.laccaria.Processes")]
impl ProcessManager {
    async fn start(&self, name: String, restart: bool, command: String) -> fdo::Result<()> {
        log::info!("Starting process \"{name}\"");
        let mut proc = ActiveProcess::create(&command, &name).into_dbus()?;

        log::info!("Started \"{name}\" with pid: {}", proc.child.id());

        let static_proc = Process {
            pid: proc.child.id(),
            status: Status::Active,
            command,
            restart,
        };

        if let Err(CompareAndSwapError {
            current: Some(_), ..
        }) = self
            .db
            .compare_and_swap(
                &name,
                None::<&[u8]>,
                Some(encode(&static_proc).into_dbus()?),
            )
            .into_dbus()?
        {
            return Err(fdo::Error::Failed(
                "Process with the same name already exists".to_string(),
            ));
        };

        log::info!("Saved process info");

        proc.attach_watcher(self.db.clone());
        if restart {
            proc.attach_restart(self.db.clone());
        }

        Ok(())
    }

    async fn kill(&mut self, name: String) -> fdo::Result<()> {
        log::info!("Killing process \"{name}\"");
        let proc = Process::get(&name, &self.db).into_dbus()?;

        if let process::Status::Dead = proc.status {
            return Err(fdo::Error::Failed("Can't kill dead process".into()));
        }

        self.sys.refresh_process((proc.pid as usize).into());
        self.sys
            .process((proc.pid as usize).into())
            .ok_or(fdo::Error::Failed("Couldn't find process".into()))?
            .kill_with(sysinfo::Signal::Kill);

        log::info!("Process \"{name}\" succesfully killed");

        Ok(())
    }

    async fn list(&mut self) -> fdo::Result<Vec<(u32, String, u32, f32, f32, bool, bool)>> {
        Ok(self
            .db
            .iter()
            .filter_map(|pair| {
                let (raw_name, raw_process) = pair.ok()?;
                let (name, process): (String, Process) = (
                    String::from_utf8(raw_name.to_vec()).unwrap(),
                    decode(&raw_process).unwrap(),
                );
                match process.status {
                    Status::Active => {
                        self.sys.refresh_process((process.pid as usize).into());
                        let sys_proc = self.sys.process((process.pid as usize).into()).unwrap();

                        Some((
                            process.pid,
                            name,
                            (sys_proc.memory() / 1048576) as u32,
                            sys_proc.cpu_usage(),
                            (sys_proc.run_time() as f32 / 60.),
                            process.restart,
                            process.status as u32 == 1,
                        ))
                    }
                    Status::Dead => Some((
                        process.pid,
                        name,
                        0,
                        0.,
                        0.,
                        process.restart,
                        process.status as u32 == 1,
                    )),
                }
            })
            .collect())
    }

    async fn delete(&self, name: String) -> fdo::Result<()> {
        log::info!("Deleting process \"{name}\"");
        let proc = Process::get(&name, &self.db).into_dbus()?;

        if let process::Status::Active = proc.status {
            return Err(fdo::Error::Failed("Cannot delete active process".into()));
        }

        self.db.remove(&name).into_dbus()?;

        Ok(())
    }

    async fn restart(&mut self, name: String, force: bool) -> fdo::Result<()> {
        log::info!("Restarting process \"{name}\"");
        let mut process_model = Process::get(&name, &self.db).into_dbus()?;
        if let Status::Active = process_model.status {
            if !force {
                return Err(fdo::Error::Failed(
                    "Cannot restart running process".to_string(),
                ));
            }
            let pid = (process_model.pid as usize).into();
            self.sys.refresh_process(pid);
            self.sys.process(pid).unwrap().kill();
        }
        let mut proc = ActiveProcess::create(&process_model.command, &name).into_dbus()?;

        process_model.pid = proc.child.id();
        process_model.status = Status::Active;

        self.db
            .insert(&name, encode(&process_model).into_dbus()?)
            .into_dbus()?;

        proc.attach_watcher(self.db.clone());
        if process_model.restart {
            proc.attach_restart(self.db.clone());
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Info)
        .init();
    std::panic::set_hook(Box::new(|e| {
        log::error!("{}", e.to_string().replace('\n', " "));
    }));

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
    let db = get_db()?;
    let mut sys = System::new();
    sys.refresh_processes();

    let process_manager = ProcessManager { db, sys };
    let _connection = ConnectionBuilder::session()?
        .name("org.laccaria.Processes")?
        .serve_at("/org/laccaria/Processes", process_manager)?
        .build()
        .await?;

    log::info!("Process manager started");

    std::future::pending::<anyhow::Result<()>>().await
}
