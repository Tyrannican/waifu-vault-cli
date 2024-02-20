mod api;
mod cli;

use cli::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.commands {
        Commands::Upload(args) => api::upload(&args)?,
        Commands::Info(args) => api::info(&args)?,
        Commands::Delete(args) => api::delete(&args)?,
        Commands::Download(args) => api::download(&args)?,
        _ => {}
    }

    Ok(())
}
