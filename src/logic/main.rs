use std::sync::Arc;

use async_channel::Sender;
use futures::{future::Either, pin_mut, select, FutureExt};

use crate::{
    crypto::keys::ArkKeys,
    net::{connection::Endpoint, message::Message},
    ui::{UiInput, UiMessage, UiReceiver, UiSender},
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

pub async fn run_logic_async(ui_receiver: UiReceiver, ui_sender: UiSender) -> Res<()> {
    let sender_clone = ui_sender.clone();
    let (filtered_receiver, buffer_task, reclaim) =
        ui_receiver.into_bufferred(|input, sender| async move {
            match input {
                UiInput::Exit => Err(Er {
                    message: "".to_owned(),
                }),
                x => {
                    sender.send(x).await?;
                    Ok(())
                }
            }
        });
    let result = select_first(
        async move {
            buffer_task.await;
            Ok(())
        },
        logic_main_loop(filtered_receiver, ui_sender),
    )
    .await;
    reclaim.ask().await;

    if let Err(Er { message }) = result {
        let _ = sender_clone.log_message(&format!("Main thread exited with error: {message}"));
    }
    let _ = sender_clone.send(UiMessage::Exit).await;
    Ok(())
}

/// Enter the game's logic
async fn logic_main_loop(mut ui_receiver: UiReceiver, ui_sender: UiSender) -> Res<()> {
    let keys = GameKeys {
        board_declaration_keys: ArkKeys::load(ui_sender.clone().into(), "keys/board_declaration"),
        field_declaration_keys: ArkKeys::load(ui_sender.clone().into(), "keys/field_declaration"),
    };

    let interrupt_filter = |msg| async move {
        match msg {
            UiInput::Esc => Err(Er {
                message: "Interrupted".to_owned(),
            }),
            _ => Ok(()),
        }
    };

    loop {
        ui_sender.send(UiMessage::MainScreen).await?;

        match ui_receiver.get().await? {
            crate::ui::UiInput::HostGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = select_first(
                    ui_receiver.consume_in_loop(interrupt_filter),
                    Endpoint::<GameMessage>::accept_incoming_connection(
                        &addr,
                        &passwd,
                        ui_sender.clone().into(),
                    ),
                )
                .await
                {
                    ui_receiver = enter_lobby(
                        ui_receiver,
                        ui_sender.clone(),
                        endpoint,
                        Player::Host,
                        keys.clone(),
                    )
                    .await?;
                }
            }
            crate::ui::UiInput::JoinGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = select_first(
                    ui_receiver.consume_in_loop(interrupt_filter),
                    Endpoint::<GameMessage>::create_connection_to(
                        &addr,
                        &passwd,
                        ui_sender.clone().into(),
                    ),
                )
                .await
                {
                    ui_receiver = enter_lobby(
                        ui_receiver,
                        ui_sender.clone(),
                        endpoint,
                        Player::Client,
                        keys.clone(),
                    )
                    .await?;
                }
            }
            UiInput::Esc => {
                return Ok(());
            }
            _ => {}
        }
    }
}

async fn enter_lobby(
    ui_receiver: UiReceiver,
    ui_sender: UiSender,
    endpoint: Endpoint<GameMessage>,
    player: Player,
    keys: GameKeys,
) -> Res<UiReceiver> {
    let (net_sender, net_receiver, net_loop_task) = endpoint.as_channel_pair();
    let net_sender_clone1 = net_sender.clone();
    let filter = {
        let counter = Arc::new(net_sender_clone1);
        move |input, sender: Sender<UiInput>| {
            let net_sender = Arc::clone(&counter);
            async move {
                match input {
                    UiInput::SendMessage(sender, info) => {
                        net_sender.send(Message::Info { sender, info }).await?;
                        Ok(())
                    }
                    UiInput::Esc => Err(Er {
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

    let (filtret_ui_input, buffer_loop_task, reclaim) = ui_receiver.into_bufferred(filter);
    let ui_sender_clone = ui_sender.clone();
    let mut game_context = GameContext {
        player,
        ui_receiver: filtret_ui_input,
        ui_sender,
        net_receiver,
        net_sender,
        keys,
    };

    let buffer_loop_task_fuse = buffer_loop_task.fuse();
    let net_loop_task_fuse = net_loop_task.fuse();
    let game_loop_fuse = game_context.game_loop().fuse();

    pin_mut!(buffer_loop_task_fuse, game_loop_fuse, net_loop_task_fuse);
    loop {
        select! {
            receiver = buffer_loop_task_fuse => {
                return Ok(receiver);
            }

            r = game_loop_fuse => {
                if let Err(e) = r {
                    ui_sender_clone.log_message(&format!("Error in the main loop: {}", e.message))?;
                    reclaim.ask().await;
                }
                else {
                    ui_sender_clone.log_message("The game ended. Interrupt to exit")?;
                }
            }

            r = net_loop_task_fuse => {
                if let Err(e) = r {
                    ui_sender_clone.log_message(&format!("Received network error: {}", e.message))?;
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
