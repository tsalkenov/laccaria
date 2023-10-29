use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct StartArgs {
    #[arg(long)]
    /// Automatically restart process
    auto_restart: bool,
    /// Process name
    name: String,
    /// Command to run process
    command: String,
}

impl StartArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        log::info!("Starting process {}", self.name);
        if self.name.is_empty() {
            anyhow::bail!("Cannnot create empty command");
        }
        proxy
            .start(&self.name, self.auto_restart, &self.command)
            .await?;

        Ok(log::info!("Process started"))
    }
}
