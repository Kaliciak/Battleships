use battleships::{gui::run_gui, logic::run_logic_with_ui, ui::cli::run_cli_ui};

fn main() {
    // run_logic_with_ui(run_cli_ui);
    run_logic_with_ui(run_gui);
    // battleships::circuit::board_declaration_circuit::generate_keys();
    // battleships::circuit::field_declaration_circuit::generate_keys();
    println!("Następna stacja: Łódź Fabryczna")
    // generate_keys();
}
