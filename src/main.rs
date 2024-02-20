mod api;
mod cli;

use cli::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Upload(args) => api::upload(&args)?,
        _ => {}
    }

    Ok(())
}
