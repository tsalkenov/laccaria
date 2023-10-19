use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct RestartArgs {
    name: String
}

impl RestartArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        proxy.restart(&self.name).await?;
        Ok(())
    }
}
