use std::{
    env, fs,
    future::Future,
    process::{ExitStatus, Stdio},
};

use anyhow::Context;
use async_std::{
    process::{Child, Command},
    task::JoinHandle,
};
use bitcode::encode;

use crate::{
    db::Db,
    process::{Process, Status},
    state::{state_dir, PROC_LOG},
};

pub struct ActiveProcess {
    pub child: Child,
    name: String,
    command: Vec<String>,
    watcher: Option<JoinHandle<anyhow::Result<ExitStatus>>>,
}

impl ActiveProcess {
    pub fn create(command: &str, name: &str) -> anyhow::Result<Self> {
        let command = shlex::split(command).context("Inavlid command string")?;

        let log_file =
            fs::File::create(state_dir().join(PROC_LOG).join(name.to_string() + ".log"))?;

        Ok(ActiveProcess {
            child: Command::new(&command[0])
                .current_dir(env::current_dir()?)
                .args(&command[1..])
                .stdout(Stdio::from(log_file.try_clone()?))
                .stderr(Stdio::from(log_file))
                .spawn()?,
            name: name.to_string(),
            command,
            watcher: None,
        })
    }

    pub fn attach_watcher(&mut self, db: Db) {
        self.watcher = Some(async_std::task::spawn({
            let status_future = self.child.status();
            let name = self.name.clone();
            async move {
                watcher(name, status_future, db).await.map_err(|e| {
                    log::error!("Process watcher failed with {e}");
                    e.context("Error from watcher")
                })
            }
        }));

        async fn watcher(
            name: String,
            status_future: impl Future<Output = Result<ExitStatus, std::io::Error>>,
            db: Db,
        ) -> anyhow::Result<ExitStatus> {
            let mut static_proc = Process::get(&name, &db)?;

            let status = status_future.await?;

            static_proc.status = Status::Dead;

            db.insert(name, encode(&static_proc)?)?;

            Ok::<_, anyhow::Error>(status)
        }
    }

    pub fn attach_restart(self, db: Db) {
        async_std::task::spawn(async move {
            if let Err(e) = restarter(self, db).await {
                log::error!("Process restarter failed with {e}");
            }
        });

        async fn restarter(mut process: ActiveProcess, db: Db) -> anyhow::Result<()> {
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
                .current_dir(env::current_dir()?)
                .args(&process.command[1..])
                .spawn()?;

            let name = process.name.clone();
            let mut static_process = Process::get(name, &db)?;

            static_process.status = Status::Active;
            static_process.pid = process.child.id();

            db.insert(&process.name, encode(&static_process)?)?;

            process.attach_watcher(db.clone());
            process.attach_restart(db.clone());
            Ok(())
        }
    }
}
