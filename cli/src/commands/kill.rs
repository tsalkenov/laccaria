use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct KillArgs {
    /// Process name
    name: String,
}

impl KillArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        proxy.kill(&self.name).await?;
        Ok(())
    }
}

