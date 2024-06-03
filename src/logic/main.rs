use async_channel::Sender;
use futures::future::Either;

use crate::{
    gui::{GuiInput, GuiMessage, GuiReceiver, GuiSender},
    net::{connection::Endpoint, message::Message},
    utils::{
        async_receiver::AsyncReceiver,
        log::Log,
        result::{Er, Res},
        threads::parallel,
    },
};

use super::{
    game_loop::{GameContext, Player},
    GameMessage,
};

pub type NetSender = Sender<Message<GameMessage>>;
pub type NetReceiver = AsyncReceiver<Message<GameMessage>>;

async fn enter_lobby(
    gui_receiver: &mut GuiReceiver,
    gui_sender: &mut GuiSender,
    endpoint: Endpoint<GameMessage>,
    player: Player,
) -> Res<()> {
    let (f, net_sender, net_receiver) = endpoint.as_channel_pair();

    async fn handle_gui_input(
        input: GuiInput,
        sender: Sender<GuiInput>,
        net_sender: NetSender,
    ) -> Res<()> {
        match input {
            GuiInput::SendMessage(sender, info) => {
                net_sender.send(Message::Info { sender, info }).await?;
                Ok(())
            }
            GuiInput::Esc => Err(Er {
                message: "Interrupt".to_owned(),
            }),
            GuiInput::Exit => Err(Er {
                message: "Interrupt".to_owned(),
            }),
            x => {
                sender.send(x).await?;
                Ok(())
            }
        }
    }
    let net_sender_clone = net_sender.clone();
    let mut gui_sender_clone = gui_sender.clone();
    if let Err(e) = parallel(
        f,
        gui_receiver.with_buffer(
            move |input, sender| handle_gui_input(input, sender, net_sender.clone()),
            move |filtered_gui_input| async move {
                GameContext {
                    gui_receiver: filtered_gui_input,
                    gui_sender: gui_sender.clone(),
                    net_receiver,
                    net_sender: net_sender_clone,
                    player,
                }
                .game_loop()
                .await?;
                Ok(())
            },
        ),
    )
    .await
    {
        gui_sender_clone.log_message(&e.message);
    }

    Ok(())
}

/// Enter the game's logic
pub async fn run_logic_async(
    gui_receiver: &mut GuiReceiver,
    gui_sender: &mut GuiSender,
) -> Res<()> {
    loop {
        gui_sender.send(GuiMessage::MainScreen).await?;

        match gui_receiver.get().await? {
            crate::gui::GuiInput::HostGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = parallel(
                    gui_receiver.consume_in_loop(|msg| async move {
                        match msg {
                            GuiInput::Esc => Err(Er {
                                message: "Interrupted".to_owned(),
                            }),
                            GuiInput::Exit => Err(Er {
                                message: "Interrupted".to_owned(),
                            }),
                            _ => Ok(()),
                        }
                    }),
                    Endpoint::<GameMessage>::accept_incoming_connection(
                        &addr,
                        &passwd,
                        gui_sender.clone().into(),
                    ),
                )
                .await
                {
                    enter_lobby(gui_receiver, gui_sender, endpoint, Player::Host).await?;
                }
            }
            crate::gui::GuiInput::JoinGame { addr, passwd } => {
                if let Ok(Either::Right(endpoint)) = parallel(
                    gui_receiver.consume_in_loop(|msg| async move {
                        match msg {
                            GuiInput::Esc => Err(Er {
                                message: "Interrupted".to_owned(),
                            }),
                            GuiInput::Exit => Err(Er {
                                message: "Interrupted".to_owned(),
                            }),
                            _ => Ok(()),
                        }
                    }),
                    Endpoint::<GameMessage>::create_connection_to(
                        &addr,
                        &passwd,
                        gui_sender.clone().into(),
                    ),
                )
                .await
                {
                    enter_lobby(gui_receiver, gui_sender, endpoint, Player::Client).await?;
                }
            }
            GuiInput::Esc => {
                return Ok(());
            }
            GuiInput::Exit => {
                return Ok(());
            }
            _ => {}
        }
    }
}
