use clap::{Parser, Subcommand};

mod bus;
mod commands;

use commands::{
    delete::DeleteArgs, kill::KillArgs, list::ListArgs, restart::RestartArgs, start::StartArgs,
};
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
    /// List all processes
    List(ListArgs),
    /// Restart saved process
    Restart(RestartArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter(Some("laccaria"), log::LevelFilter::Info)
        .init();

    let cli = Cli::parse();

    let connection = Connection::session().await?;
    let proxy = bus::ProcessManagerProxy::new(&connection).await?;

    if let Err(e) = match cli.command {
        Commands::Start(args) => args.run(proxy).await,
        Commands::Kill(args) => args.run(proxy).await,
        Commands::Delete(args) => args.run(proxy).await,
        Commands::List(args) => args.run(proxy).await,
        Commands::Restart(args) => args.run(proxy).await,
    } {
        log::error!("Error during command execution:\n{e}");
    }

    Ok(())
}
