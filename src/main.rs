use battleships::{logic::run_logic_with_ui, ui::cli::run_cli, ui::gui::run_gui};
use clap::{Parser, Subcommand};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Command::GenerateKeys) => {
            battleships::circuit::board_declaration_circuit::generate_keys();
            battleships::circuit::field_declaration_circuit::generate_keys();
        }
		Some(Command::Gui) => {
			run_logic_with_ui(run_gui);
		}
		Some(Command::Cli) => {
			run_logic_with_ui(run_cli);
		}
        None => {
            run_logic_with_ui(run_gui);
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
    GenerateKeys,
	Gui,
	Cli,
}
