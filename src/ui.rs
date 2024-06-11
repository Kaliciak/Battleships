use async_channel::Sender;

pub mod cli;
pub mod gui;

use crate::{
    logic::GameState,
    model::{IncompleteBoard, Ship},
    utils::{
        async_receiver::AsyncReceiver,
        log::{Log, Logger},
        result::Res,
    },
};

pub type UiSender = Sender<UiMessage>;
pub type UiReceiver = AsyncReceiver<UiInput>;

impl Log for UiSender {
    fn log_message(&self, msg: &str) -> Res<()> {
        self.send_blocking(UiMessage::Log(msg.to_owned()))?;
        Ok(())
    }
}

impl From<UiSender> for Logger {
    fn from(value: UiSender) -> Self {
        Logger::new(value)
    }
}

/// Message (state) that can be send to the UI
#[derive(Clone)]
pub enum UiMessage {
    Log(String),
    MainScreen,
    Lobby,
    BoardConstruction(IncompleteBoard),
    PrintGameState(GameState),
    Exit,
}

/// Input received from the UI
pub enum UiInput {
    HostGame { addr: String, passwd: String },
    JoinGame { addr: String, passwd: String },
    SendMessage(String, String),
    PutShip(Ship),
    Shoot(u8, u8),
    Esc,
    Exit,
}
