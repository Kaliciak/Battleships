use battleships::{gui::cli::run_cli_gui, logic::run_logic_with_gui};
use clap::{Parser, Subcommand};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::GenerateKeys) => {
            battleships::circuit::board_declaration_circuit::generate_keys();
            battleships::circuit::field_declaration_circuit::generate_keys();
        }
        None => {
            run_logic_with_gui(run_cli_gui);
        }
    }
    println!("Następna stacja: Łódź Fabryczna")
}

#[derive(Debug, Parser)]
#[clap(name = "cli", version)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    GenerateKeys
}