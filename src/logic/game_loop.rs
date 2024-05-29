use futures::{pin_mut, select, FutureExt};

use crate::{
    gui::{Gui, Input},
    networking::{connection::Endpoint, message::Message},
};

use super::GameMessage;

enum LoopInput {
    GuiInput(Input),
    NetworkInput(Message<GameMessage>),
}

async fn get_loop_input(channel: &mut Endpoint<GameMessage>, gui: &mut impl Gui) -> LoopInput {
    let mut logger = gui.get_logger();
    let gfut = gui.receive_input().fuse();
    let nfut = channel.receive().fuse();

    pin_mut!(gfut, nfut);

    select! {
        ginp = gfut => LoopInput::GuiInput(ginp),
        nval = nfut =>
            match nval {
                Ok(ninp) => LoopInput::NetworkInput(ninp),
                Err(e) => {
                    logger.log_message(&e.message);
                    LoopInput::GuiInput(Input::Esc)
                },
            }

    }
}

pub async fn game_loop(mut channel: Endpoint<GameMessage>, gui: &mut impl Gui) {
    let mut logger = gui.get_logger();
    logger.log_message("Entering main game loop");
    gui.show_lobby();
    loop {
        match get_loop_input(&mut channel, gui).await {
            LoopInput::GuiInput(gui_input) => match gui_input {
                crate::gui::Input::Esc => {
                    return;
                }
                crate::gui::Input::Exit => {
                    return;
                }
                crate::gui::Input::SendMessage(sender, info) => {
                    if let Err(e) = channel.send(&Message::Info { sender, info }).await {
                        logger.log_message(&e.message);
                        return;
                    }
                }
                _ => logger.log_message("Illegal command"),
            },
            LoopInput::NetworkInput(message) => match message {
                Message::Info { sender, info } => {
                    logger.log_message(&format!("{}|  {}> {}", channel.second_addr, sender, info))
                }
                Message::Error { sender, info } => logger.log_message(&format!(
                    "{}|  {}!!!> {}",
                    channel.second_addr, sender, info
                )),
                Message::Value(_) => todo!(),
            },
        }
    }
}
