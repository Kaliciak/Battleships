use futures::{pin_mut, select, FutureExt};

use crate::{
    gui::{get_gui_input, Gui, Input},
    net::{
        connection::{get_net_input, Endpoint},
        message::Message,
    },
    utils::{input::InputFilter, log::Log},
};

use super::GameMessage;

enum LoopInput {
    GuiInput(Input),
    NetworkInput(Message<GameMessage>),
}

async fn get_loop_input(
    filter: &mut InputFilter,
    channel: &mut Endpoint<GameMessage>,
    gui: &mut impl Gui,
) -> LoopInput {
    let mut filter_copy = filter.clone();
    let gfut = get_gui_input(filter, gui).fuse();
    let nfut = get_net_input(&mut filter_copy, channel).fuse();

    pin_mut!(gfut, nfut);

    select! {
        ginp = gfut => LoopInput::GuiInput(ginp),
        nval = nfut => LoopInput::NetworkInput(nval)
    }
}

pub async fn game_loop(
    filter: &mut InputFilter,
    mut channel: Endpoint<GameMessage>,
    gui: &mut impl Gui,
) {
    let mut logger = gui.get_logger();
    gui.show_lobby();
    loop {
        match get_loop_input(filter, &mut channel, gui).await {
            LoopInput::GuiInput(gui_input) => match gui_input {
                crate::gui::Input::SendMessage(sender, info) => {
                    if let Err(e) = channel.send(&Message::Info { sender, info }).await {
                        logger.log_message(&e.message);
                        return;
                    }
                }
                _ => logger.log_message("Illegal command"),
            },
            _ => {}
        }
    }
}
