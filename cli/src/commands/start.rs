use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct StartArgs {
    /// Process name
    name: String,
    /// Command to run process
    command: String,
}

impl StartArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        let Some(args) = shlex::split(&self.command) else {
            log::error!("Cannot parse invalid command");
            std::process::exit(1);
        };

        proxy.start(&self.name, args).await?;
        Ok(())
    }
}
