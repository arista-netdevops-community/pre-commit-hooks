use std::process::ExitCode;

use clap::Parser;

#[path = "../about.rs"]
mod about;

#[derive(Parser)]
#[command(about = "Generate Rust dependency license information using cargo-about")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Render dependency licenses using a single Handlebars template.
    Generate(about::Args),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let Command::Generate(args) = cli.command;
    match about::generate(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
