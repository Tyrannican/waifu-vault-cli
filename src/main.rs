mod cli;

use cli::*;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("CLI: {cli:?}");

    println!("Hello, world!");
}
