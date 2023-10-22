use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct KillArgs {
    /// Process name
    name: String,
}

impl KillArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        log::info!("Stopping process {}", self.name);
        proxy.kill(&self.name).await?;

        Ok(log::info!("Process stopped"))
    }
}
