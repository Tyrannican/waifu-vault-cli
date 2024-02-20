mod api;
mod cli;

use cli::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("CLI: {cli:?}");
}
