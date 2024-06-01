use logic::run_main_loop_with_cli;

mod circuit;
mod gui;
mod logic;
mod model;
mod net;

fn main() {
    run_main_loop_with_cli();
    println!("Następna stacja: Łódź Fabryczna")
}
