mod cli;
mod commands;

use cli::*;
use commands::{delete_file, download_file, file_info, modify_file, upload_file};

use anyhow::{Context, Result};
use waifuvault::ApiCaller;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let caller = ApiCaller::new();

    match cli.commands {
        Commands::Upload(args) => upload_file(args, caller).await.context("upload request")?,
        Commands::Info { token } => file_info(token, caller)
            .await
            .context("file info request")?,
        Commands::Modify(args) => modify_file(args, caller)
            .await
            .context("update file request")?,
        Commands::Delete { token } => delete_file(token, caller)
            .await
            .context("delete file request")?,
        Commands::Download(args) => download_file(args, caller)
            .await
            .context("download file request")?,
    }

    Ok(())
}
