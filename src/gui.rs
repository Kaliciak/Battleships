pub trait Logger {
    fn log_message(&mut self, msg: &str);
}

pub trait Gui {
    fn get_logger(&mut self) -> Box<dyn Logger>;
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
