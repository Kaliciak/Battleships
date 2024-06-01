use crate::utils::{input::InputFilter, log::Logger};

pub trait Gui {
    fn get_logger(&mut self) -> Logger;
    async fn receive_input(&mut self) -> Input;
    fn go_to_main_screen(&mut self);
    fn show_lobby(&mut self);
}

pub enum Input {
    HostGame { addr: String, passwd: String },
    JoinGame { addr: String, passwd: String },
    SendMessage(String, String),
    Esc,
    Exit,
}

pub async fn get_gui_input(filter: &mut InputFilter, gui: &mut impl Gui) -> Input {
    match gui.receive_input().await {
        Input::Esc => filter.interrupt().await,
        Input::Exit => filter.exit().await,
        x => x,
    }
}
