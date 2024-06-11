use battleships::{logic::run_logic_with_ui, ui::cli::run_cli, ui::gui::run_gui};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Commands {
    #[command(subcommand)]
    ui: Option<UI>
}

#[derive(Subcommand)]
enum UI {
    Gui,
    Cli
}

fn main() {
    let args = Commands::parse();
    match args.ui {
        Some(UI::Gui) => run_logic_with_ui(run_gui),
        Some(UI::Cli) => run_logic_with_ui(run_cli),
        None => run_logic_with_ui(run_gui)
    }
    // battleships::circuit::board_declaration_circuit::generate_keys();
    // battleships::circuit::field_declaration_circuit::generate_keys();
    println!("Następna stacja: Łódź Fabryczna")
    // generate_keys();
}
