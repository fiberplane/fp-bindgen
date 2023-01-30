mod clean;
mod test;
mod utils;

use clap::{Parser, Subcommand};
use console::{style, Emoji};

type TaskResult<T> = anyhow::Result<T>;

static ERROR: Emoji<'_, '_> = Emoji("🤒 ", "");

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Cleans all target folders
    Clean,
    /// Builds test protocol and plugin and runs all available tests
    Test,
}

fn main() {
    if let Err(e) = handle_cli() {
        println!("      {}{}", ERROR, style(format!("Error: {e}")).red());
    }
}

fn handle_cli() -> TaskResult<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Clean) => clean::clean()?,
        Some(Commands::Test) => test::test()?,
        None => {}
    }

    Ok(())
}
