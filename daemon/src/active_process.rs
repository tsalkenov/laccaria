use std::{future::Future, process::ExitStatus};

use anyhow::Context;
use async_std::{
    process::{Child, Command},
    task::JoinHandle,
};
use sea_orm::{ActiveModelTrait, DbConn, IntoActiveModel, Set};

use crate::process;

pub struct ActiveProcess {
    pub child: Child,
    name: String,
    command: Vec<String>,
    watcher: Option<JoinHandle<anyhow::Result<ExitStatus>>>,
}

impl ActiveProcess {
    pub fn create(command: &str, name: String) -> anyhow::Result<Self> {
        let command = shlex::split(command).context("Inavlid command string")?;

        Ok(ActiveProcess {
            child: Command::new(&command[0]).args(&command[1..]).spawn()?,
            name,
            command,
            watcher: None,
        })
    }

    pub fn attach_watcher(&mut self, db: DbConn) {
        let status_future = self.child.status();
        let pid = self.child.id();
        self.watcher = Some(async_std::task::spawn(async move {
            watcher(pid, status_future, db).await.map_err(|e| {
                log::error!("Process watcher failed with {e}");
                e.context("Error from watcher")
            })
        }));

        async fn watcher(
            pid: u32,
            status_future: impl Future<Output = Result<ExitStatus, std::io::Error>>,
            db: DbConn,
        ) -> anyhow::Result<ExitStatus> {
            let mut process_model = process::Process::find_by_pid(pid, &db)
                .await?
                .into_active_model();

            let status = status_future.await?;

            process_model.status = Set(process::Status::Dead);
            process_model.update(&db).await?;

            Ok::<_, anyhow::Error>(status)
        }
    }

    pub fn attach_restart(self, db: DbConn) {
        async_std::task::spawn(async move {
            if let Err(e) = restarter(self, db).await {
                log::error!("Process restarter failed with {e}");
            }
        });

        async fn restarter(mut process: ActiveProcess, db: DbConn) -> anyhow::Result<()> {
            let status = process
                .watcher
                .context("Cannot attach restart to unwatched process")?
                .await?;

            if status.code().unwrap_or(0) == 0 {
                return Ok(());
            }

            log::warn!(
                "Process {} failed with exit code {}",
                process.name,
                status.code().unwrap_or(1)
            );

            process.watcher = None;

            process.child = Command::new(&process.command[0])
                .args(&process.command[1..])
                .spawn()?;

            let name = process.name.clone();
            let mut process_model = process::Process::find_by_name(&name, &db)
                .await?
                .into_active_model();

            process_model.status = Set(process::Status::Active);
            process_model.pid = Set(process.child.id());
            process_model.update(&db).await?;

            process.attach_watcher(db.clone());

            process.attach_restart(db.clone());
            Ok(())
        }
    }
}
