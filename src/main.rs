mod generate;
mod init;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::Term;
use dialoguer::theme::ColorfulTheme;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init(init::InitArgs),
    Generate(generate::GenerateArgs),
    Add,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let term = Term::buffered_stderr();
    let theme = ColorfulTheme::default();

    match args.cmd {
        Some(Commands::Generate(args)) => return generate::generate_files(args, term, theme),
        Some(Commands::Init(args)) => return init::init_starter(args, term, theme),
        _ => {}
    };

    Ok(())
}
