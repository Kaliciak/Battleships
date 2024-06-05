use logic::run_main_loop_with_cli;

mod circuit;
mod gui;
mod logic;
mod model;
mod net;
mod ui;

fn main() {
    //run_main_loop_with_cli();
    gui::launch_gui();
    println!("Następna stacja: Łódź Fabryczna")
}
