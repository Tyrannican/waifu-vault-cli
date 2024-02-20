pub use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Upload a file to vault
    Upload(UploadArgs),

    /// Retrieve a file from the vault
    Download(DownloadArgs),

    /// Information about a file in the vault
    Info(InfoArgs),

    /// Delete a file from the vault
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct UploadArgs {
    /// File to upload to the Vault (Max size 100MB)
    #[arg(short, long, conflicts_with = "url", required_unless_present = "url")]
    file: Option<String>,

    /// Upload to the Vault via a URL (Max size 100MB)
    #[arg(short, long, required_unless_present = "file")]
    url: Option<String>,

    /// Set an expiry time for the content (e.g. 1d for 1 day (m for minutes, h for hours))
    #[arg(short, long)]
    expiry: Option<String>,

    /// Hide the filename from the generated URL
    #[arg(long)]
    hide_filename: bool,

    /// Set a password for the file which is required when downloading the file
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Download token generated when the file was uploaded
    #[arg(short, long)]
    token: String,

    /// Where to store the output file
    #[arg(short, long, default_value_t = String::from("."))]
    output: String,

    /// Password if required to download the file
    #[arg(short, long)]
    password: Option<String>,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Token generated from when the file was uploaded
    #[arg(short, long)]
    token: String,

    /// Show human-readable timestamp for the file expiry
    #[arg(short, long)]
    formatted: bool,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Token generated from when the file was uploaded
    #[arg(short, long)]
    token: String,
}
