use std::sync::Arc;

use async_channel::Sender;
use futures::{future::Either, pin_mut, select, FutureExt};

use crate::{
    crypto::keys::ArkKeys,
    gui::{GuiInput, GuiMessage, GuiReceiver, GuiSender},
    net::{connection::Endpoint, message::Message},
    utils::{
        async_receiver::AsyncReceiver,
        log::Log,
        result::{Er, Res},
        threads::select_first,
    },
};

use super::{
    game_loop::{GameContext, Player},
    GameMessage,
};

pub type NetSender = Sender<Message<GameMessage>>;
pub type NetReceiver = AsyncReceiver<Message<GameMessage>>;

pub async fn run_logic_async(gui_receiver: GuiReceiver, gui_sender: GuiSender) -> Res<()> {
    let (filtered_receiver, buffer_task, reclaim) =
        gui_receiver.into_bufferred(|input, sender| async move {
            match input {
                GuiInput::Exit => Err(Er {
                    message: "".to_owned(),
                }),
                x => {
                    sender.send(x).await?;
                    Ok(())
                }
            }
        });
    select_first(
        async move {
            buffer_task.await;
            Ok(())
        },
        logic_main_loop(filtered_receiver, gui_sender),
    )
    .await?;
    reclaim.ask().await;
    Ok(())
}

/// Enter the game's logic
async fn logic_main_loop(mut gui_receiver: GuiReceiver, gui_sender: GuiSender) -> Res<()> {
    let keys = GameKeys {
        board_declaration_keys: ArkKeys::load(gui_sender.clone().into(), "keys/board_declaration"),
        field_declaration_keys: ArkKeys::load(gui_sender.clone().into(), "keys/field_declaration"),
    };

    let interrupt_filter = |msg| async move {
        match msg {
            GuiInput::Esc => Err(Er {
                message: "Interrupted".to_owned(),
            }),
            _ => Ok(()),
        }
    };

    loop {
        gui_sender.send(GuiMessage::MainScreen).await?;

        match gui_receiver.get().await? {
            crate::gui::GuiInput::HostGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = select_first(
                    gui_receiver.consume_in_loop(interrupt_filter),
                    Endpoint::<GameMessage>::accept_incoming_connection(
                        &addr,
                        &passwd,
                        gui_sender.clone().into(),
                    ),
                )
                .await
                {
                    gui_receiver = enter_lobby(
                        gui_receiver,
                        gui_sender.clone(),
                        endpoint,
                        Player::Host,
                        keys.clone(),
                    )
                    .await?;
                }
            }
            crate::gui::GuiInput::JoinGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = select_first(
                    gui_receiver.consume_in_loop(interrupt_filter),
                    Endpoint::<GameMessage>::create_connection_to(
                        &addr,
                        &passwd,
                        gui_sender.clone().into(),
                    ),
                )
                .await
                {
                    gui_receiver = enter_lobby(
                        gui_receiver,
                        gui_sender.clone(),
                        endpoint,
                        Player::Client,
                        keys.clone(),
                    )
                    .await?;
                }
            }
            GuiInput::Esc => {
                return Ok(());
            }
            _ => {}
        }
    }
}

async fn enter_lobby(
    gui_receiver: GuiReceiver,
    gui_sender: GuiSender,
    endpoint: Endpoint<GameMessage>,
    player: Player,
    keys: GameKeys,
) -> Res<GuiReceiver> {
    let (net_sender, net_receiver, net_loop_task) = endpoint.as_channel_pair();
    let net_sender_clone1 = net_sender.clone();
    let filter = {
        let counter = Arc::new(net_sender_clone1);
        move |input, sender: Sender<GuiInput>| {
            let net_sender = Arc::clone(&counter);
            async move {
                match input {
                    GuiInput::SendMessage(sender, info) => {
                        net_sender.send(Message::Info { sender, info }).await?;
                        Ok(())
                    }
                    GuiInput::Esc => Err(Er {
                        message: "Interrupt".to_owned(),
                    }),
                    x => {
                        sender.send(x).await?;
                        Ok(())
                    }
                }
            }
        }
    };

    let (filtret_gui_input, buffer_loop_task, reclaim) = gui_receiver.into_bufferred(filter);
    let gui_sender_clone = gui_sender.clone();
    let mut game_context = GameContext {
        player,
        gui_receiver: filtret_gui_input,
        gui_sender,
        net_receiver,
        net_sender,
        keys,
    };

    let buffer_loop_task_fuse = buffer_loop_task.fuse();
    let net_loop_task_fuse = net_loop_task.fuse();
    let game_loop_fuse = game_context.game_loop().fuse();

    pin_mut!(buffer_loop_task_fuse, game_loop_fuse, net_loop_task_fuse);
    let mut main_loop_finished = false;
    loop {
        select! {
            receiver = buffer_loop_task_fuse => {
                if !main_loop_finished {
                    if let Err(e) = game_loop_fuse.await {
                    gui_sender_clone.log_message(&format!("Error in the main loop: {}", e.message))?;
                    }
                }
                return Ok(receiver);
            }

            r = game_loop_fuse => {
                if let Err(e) = r {
                    gui_sender_clone.log_message(&format!("Error in the main loop: {}", e.message))?;
                }
                main_loop_finished = true;
                reclaim.ask().await;
            }

            r = net_loop_task_fuse => {
                if let Err(e) = r {
                    gui_sender_clone.log_message(&format!("Received network error: {}", e.message))?;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameKeys {
    pub board_declaration_keys: ArkKeys,
    pub field_declaration_keys: ArkKeys,
}
