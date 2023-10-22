use clap::Args;
use comfy_table::{
    modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
    presets::UTF8_FULL,
    Cell, Color,
};

use crate::bus::ProcessManagerProxy;

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long)]
    /// Filter dead processes
    only_active: bool,
}

impl ListArgs {
    pub async fn run(self, proxy: ProcessManagerProxy<'_>) -> anyhow::Result<()> {
        log::info!("Listing saved processes");
        let procs = proxy.list().await?;

        let mut rows = vec![];
        for (pid, name, memory, cpu, run_time, auto_restart, status) in procs {
            if self.only_active && !status {
                continue;
            }
            rows.push([
                pid.to_string(),
                name,
                if status {
                    "online".to_string()
                } else {
                    "offline".to_string()
                },
                format!("{}%", cpu),
                format!("{}-Mb", memory),
                format!("{:.2} m", run_time),
                if auto_restart {
                    "enabled".to_string()
                } else {
                    "disabled".to_string()
                },
            ])
        }
        if rows.is_empty() {
            log::info!("Processes not found");
            std::process::exit(0);
        }
        let mut table = comfy_table::Table::new();
        table
            .set_header(
                [
                    "pid",
                    "name",
                    "status",
                    "cpu",
                    "ram",
                    "uptime",
                    "auto-restart",
                ]
                .iter()
                .map(|i| Cell::from(i).fg(Color::Red))
                .collect::<Vec<Cell>>(),
            )
            .add_rows(rows)
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS);

        Ok(println!("{}", table))
    }
}
