use async_channel::Sender;

use crate::{
    logic::GameState,
    model::{IncompleteBoard, Ship},
    utils::{
        async_receiver::AsyncReceiver,
        log::{Log, Logger},
        result::Res,
    },
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
    PrintGameState(GameState),
}

/// Input received from the GUI
pub enum GuiInput {
    HostGame { addr: String, passwd: String },
    JoinGame { addr: String, passwd: String },
    SendMessage(String, String),
    PutShip(Ship),
    Shoot(u8, u8),
    Esc,
    Exit,
}
