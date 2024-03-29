//! Main CLI arguments and types

pub use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Cli to interact with the Waifu Vault API
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Upload a file to vault
    Upload(UploadArgs),

    /// Retrieve a file from the vault
    Download(DownloadArgs),

    /// Information about a file in the vault
    Info(TokenArgs),

    /// Delete a file from the vault
    Delete(TokenArgs),
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    /// File to upload to the Vault (Max size 100MB)
    #[arg(short, long, conflicts_with = "url", required_unless_present = "url")]
    pub file: Option<String>,

    /// Upload to the Vault via a URL (Max size 100MB)
    #[arg(short, long, required_unless_present = "file")]
    pub url: Option<String>,

    /// Set an expiry time for the content (e.g. 1d for 1 day (m for minutes, h for hours))
    #[arg(short, long)]
    pub expires: Option<String>,

    /// Hide the filename from the generated URL
    #[arg(long)]
    pub hide_filename: bool,

    /// Set a password for the file which is required when downloading the file
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// URL of the item to download
    #[arg(short, long)]
    pub url: String,

    /// Output file
    #[arg(short, long)]
    pub output: Option<String>,

    /// Password if required to download the file
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Args, Debug)]
pub struct TokenArgs {
    /// Token generated from when the file was uploaded
    #[arg(short, long)]
    pub token: String,
}
