use futures::{pin_mut, select, FutureExt};

use crate::{
    ui::{UI, Input},
    net::{connection::Endpoint, message::Message},
};

use super::GameMessage;

enum LoopInput {
    UiInput(Input),
    NetworkInput(Message<GameMessage>),
}

async fn get_loop_input(channel: &mut Endpoint<GameMessage>, ui: &mut impl UI) -> LoopInput {
    let mut logger = ui.get_logger();
    let gfut = ui.receive_input().fuse();
    let nfut = channel.receive().fuse();

    pin_mut!(gfut, nfut);

    select! {
        ginp = gfut => LoopInput::UiInput(ginp),
        nval = nfut =>
            match nval {
                Ok(ninp) => LoopInput::NetworkInput(ninp),
                Err(e) => {
                    logger.log_message(&e.message);
                    LoopInput::UiInput(Input::Esc)
                },
            }

    }
}

pub async fn game_loop(mut channel: Endpoint<GameMessage>, ui: &mut impl UI) {
    let mut logger = ui.get_logger();
    logger.log_message("Entering main game loop");
    ui.show_lobby();
    loop {
        match get_loop_input(&mut channel, ui).await {
            LoopInput::UiInput(ui_input) => match ui_input {
                crate::ui::Input::Esc => {
                    return;
                }
                crate::ui::Input::Exit => {
                    return;
                }
                crate::ui::Input::SendMessage(sender, info) => {
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
