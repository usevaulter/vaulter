use clap::Parser;
use vaulter::cli::Cli;
use vaulter::commands;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = commands::run(cli.command).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
