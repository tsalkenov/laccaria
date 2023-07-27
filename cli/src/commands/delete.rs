use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct DeleteArgs {
    name: String
}

impl DeleteArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        proxy.delete(&self.name).await?;
        Ok(())
    }
}
