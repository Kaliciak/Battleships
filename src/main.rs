use battleships::{gui::cli::run_cli_gui, logic::run_logic_with_gui};

fn main() {
    run_logic_with_gui(run_cli_gui);
    // battleships::circuit::board_declaration_circuit::generate_keys();
    // battleships::circuit::field_declaration_circuit::generate_keys();
    println!("Następna stacja: Łódź Fabryczna")
    // generate_keys();
}
