use clap::Args;

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct RestartArgs {
    name: String,
    #[arg(long)]
    /// Restart even if process is running
    force: bool
}

impl RestartArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        proxy.restart(&self.name, self.force).await?;
        Ok(())
    }
}
