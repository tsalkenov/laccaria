use clap::{Parser, Subcommand};

mod bus;
mod commands;

use commands::{delete::DeleteArgs, kill::KillArgs, start::StartArgs};
use zbus::Connection;

#[derive(Parser)]
#[command(name = "Laccaria")]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start process by giving it name and command
    Start(StartArgs),
    /// Kill process
    Kill(KillArgs),
    /// Permanently remove any state conne related to process
    Delete(DeleteArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let connection = Connection::session().await?;
    let proxy = bus::ProcessManagerProxy::new(&connection).await?;

    match cli.command {
        Commands::Start(args) => args.run(proxy).await,
        Commands::Kill(args) => args.run(proxy).await,
        Commands::Delete(args) => args.run(proxy).await,
    }
}
