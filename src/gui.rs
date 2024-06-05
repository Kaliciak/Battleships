use async_channel::Sender;

use crate::{
    model::IncompleteBoard,
    utils::{
        async_receiver::AsyncReceiver,
        log::{Log, Logger},
        result::Res,
    },
    Ship,
};

pub type GuiSender = Sender<GuiMessage>;
pub type GuiReceiver = AsyncReceiver<GuiInput>;

impl Log for GuiSender {
    fn log_message(&self, msg: &str) -> Res<()> {
        self.send_blocking(GuiMessage::Log(msg.to_owned()))?;
        Ok(())
    }
}

impl From<GuiSender> for Logger {
    fn from(value: GuiSender) -> Self {
        Logger::new(value)
    }
}

/// Message (state) that can be send to the GUI
pub enum GuiMessage {
    Log(String),
    MainScreen,
    Lobby,
    BoardConstruction(IncompleteBoard),
}

/// Input received from the GUI
pub enum GuiInput {
    HostGame { addr: String, passwd: String },
    JoinGame { addr: String, passwd: String },
    SendMessage(String, String),
    PutShip(Ship),
    Esc,
    Exit,
}
