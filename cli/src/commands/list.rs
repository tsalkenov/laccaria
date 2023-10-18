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
        let procs = proxy.list().await?;

        let mut rows = vec![];
        for (pid, name, memory, cpu, run_time, status) in procs {
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
                format!("{} m", run_time),
            ])
        }
        let mut table = comfy_table::Table::new();
        table
            .set_header(
                ["pid", "name", "status", "cpu", "ram", "uptime"]
                    .iter()
                    .map(|i| Cell::from(i).fg(Color::Red))
                    .collect::<Vec<Cell>>(),
            )
            .add_rows(rows)
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS);

        println!("{}", table);

        Ok(())
    }
}
