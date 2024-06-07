use crate::{
    circuit::board_declaration_circuit::BoardDeclarationCircuit,
    crypto::{keys::ArkKeys, proofs::CorrectnessProof},
    gui::{GuiInput, GuiMessage, GuiReceiver, GuiSender},
    model::{Board, IncompleteBoard},
    net::message::Message,
    utils::{
        log::{Log, Logger},
        result::{Er, Res},
        threads::{merge, spawn_thread_async},
    },
};

use super::{
    game_loop::GameContext,
    main::{NetReceiver, NetSender},
    GameMessage,
};

/// Build a correct board according to the user inputs
async fn build_board(gui_receiver: &mut GuiReceiver, gui_sender: GuiSender) -> Res<Board> {
    let mut inc_board = IncompleteBoard::new();
    while inc_board.0.len() < 15 {
        gui_sender
            .send(GuiMessage::BoardConstruction(inc_board.clone()))
            .await?;

        if let GuiInput::PutShip(ship) = gui_receiver.get().await? {
            if !inc_board.can_be_extended_with(ship) {
                gui_sender.log_message("Cannot extend the board with this ship")?;
            } else {
                gui_sender.log_message("Ship successfully added")?;
                inc_board.extend(ship);
            }
        }
    }

    gui_sender.log_message("Board has been successfully build!")?;
    Ok(inc_board.build_board())
}

/// Build board, generate proof and send it to the other player
async fn build_and_prove_board(
    gui_receiver: &mut GuiReceiver,
    gui_sender: GuiSender,
    net_sender: NetSender,
    keys: ArkKeys,
) -> Res<BoardDeclarationCircuit> {
    let circ: BoardDeclarationCircuit = build_board(gui_receiver, gui_sender.clone()).await?.into();
    // let circ: BoardDeclarationCircuit = SAMPLE_BOARD.into();
    // let board = SAMPLE_BOARD;

    gui_sender.log_message("Generating board correctness proof. This can take a while...")?;

    let logger: Logger = gui_sender.clone().into();
    let proof = spawn_thread_async(move || CorrectnessProof::create(circ, logger, keys)).await??;

    gui_sender.log_message(&format!(
        "Successfully generated board correctness proof. Salt: {:?} Hash: {:?}",
        circ.salt, circ.hash
    ))?;

    net_sender
        .send(Message::Value(GameMessage::BoardIsCorrect(
            proof, circ.hash,
        )))
        .await?;

    gui_sender.log_message("Proof has been sent to the other player.")?;

    Ok(circ)
}

/// Receive and verify other player's proof
async fn receive_and_verify_board_proof(
    net_receiver: &mut NetReceiver,
    gui_sender: &mut GuiSender,
    keys: ArkKeys,
) -> Res<[u8; 32]> {
    loop {
        if let Message::Value(GameMessage::BoardIsCorrect(mut proof, hash)) =
            net_receiver.get().await?
        {
            gui_sender.log_message(&format!(
                "Received board correctness proof from the other player. Hash {:?}.\nVerifying received proof...",
                hash
            ))?;

            let hash_clone = hash.to_vec();
            if spawn_thread_async(move || proof.is_correct(hash_clone.to_vec().into(), keys))
                .await??
            {
                gui_sender.log_message("Received proof is correct!")?;
                return Ok(hash);
            } else {
                gui_sender.log_message("Invalid proof")?;
                return Err(Er {
                    message: "Invalid proof".to_owned(),
                });
            }
        }
    }
}

/// Handle boards creation and verification.
/// Returns constructed board and the hash of the board of the other player
pub async fn initialize_boards(
    game_context: &mut GameContext,
) -> Res<(BoardDeclarationCircuit, [u8; 32])> {
    if let Ok((board, hash)) = merge(
        build_and_prove_board(
            &mut game_context.gui_receiver,
            game_context.gui_sender.clone(),
            game_context.net_sender.clone(),
            game_context.keys.board_declaration_keys.clone(),
        ),
        receive_and_verify_board_proof(
            &mut game_context.net_receiver,
            &mut game_context.gui_sender,
            game_context.keys.board_declaration_keys.clone(),
        ),
    )
    .await
    {
        Ok((board, hash))
    } else {
        Err(Er {
            message: "Error while initializing boards".to_owned(),
        })
    }
}
