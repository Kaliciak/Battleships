use crate::{
    gui::{GuiReceiver, GuiSender},
    utils::{log::Log, result::Res},
};

use super::{
    board_creation::initialize_boards,
    main::{NetReceiver, NetSender},
};

pub enum Player {
    Client,
    Host,
}

/// All relevant information/channels available in the main game loop.
pub struct GameContext {
    pub player: Player,
    pub gui_receiver: GuiReceiver,
    pub gui_sender: GuiSender,
    pub net_receiver: NetReceiver,
    pub net_sender: NetSender,
}

impl GameContext {
    pub async fn game_loop(&mut self) -> Res<()> {
        self.gui_sender.send(crate::gui::GuiMessage::Lobby).await?;
        initialize_boards(self).await?;
        self.gui_sender
            .log_message("Boards has been successfully initialized!")?;
        Ok(())
    }
}
