mod api;
mod cli;

use cli::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut api_caller = api::ApiCaller::new();

    match &cli.commands {
        Commands::Upload(args) => api_caller.upload(args)?,
        Commands::Info(args) => api_caller.info(args)?,
        Commands::Delete(args) => api_caller.delete(args)?,
        Commands::Download(args) => api_caller.download(args)?,
    }

    Ok(())
}
