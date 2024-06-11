use crate::{
    circuit::board_declaration_circuit::BoardDeclarationCircuit,
    crypto::{keys::ArkKeys, proofs::CorrectnessProof},
    ui::{UiInput, UiMessage, UiReceiver, UiSender},
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
async fn build_board(ui_receiver: &mut UiReceiver, ui_sender: UiSender) -> Res<Board> {
    let mut inc_board = IncompleteBoard::new();
    while inc_board.0.len() < 15 {
        ui_sender
            .send(UiMessage::BoardConstruction(inc_board.clone()))
            .await?;

        if let UiInput::PutShip(ship) = ui_receiver.get().await? {
            if !inc_board.can_be_extended_with(ship) {
                ui_sender.log_message("Cannot extend the board with this ship")?;
            } else {
                ui_sender.log_message("Ship successfully added")?;
                inc_board.extend(ship);
            }
        }
    }

    ui_sender.log_message("Board has been successfully build!")?;
    Ok(inc_board.build_board())
}

/// Build board, generate proof and send it to the other player
async fn build_and_prove_board(
    ui_receiver: &mut UiReceiver,
    ui_sender: UiSender,
    net_sender: NetSender,
    keys: ArkKeys,
) -> Res<BoardDeclarationCircuit> {
    let circ: BoardDeclarationCircuit = build_board(ui_receiver, ui_sender.clone()).await?.into();
    // let circ: BoardDeclarationCircuit = SAMPLE_BOARD.into();
    // let board = SAMPLE_BOARD;

    ui_sender.log_message("Generating board correctness proof. This can take a while...")?;

    let logger: Logger = ui_sender.clone().into();
    let proof = spawn_thread_async(move || CorrectnessProof::create(circ, logger, keys)).await??;

    ui_sender.log_message(&format!(
        "Successfully generated board correctness proof. Salt: {:?} Hash: {:?}",
        circ.salt, circ.hash
    ))?;

    net_sender
        .send(Message::Value(GameMessage::BoardDeclaration(
            proof, circ.hash,
        )))
        .await?;

    ui_sender.log_message("Proof has been sent to the other player.")?;

    Ok(circ)
}

/// Receive and verify other player's proof
async fn receive_and_verify_board_proof(
    net_receiver: &mut NetReceiver,
    ui_sender: &mut UiSender,
    keys: ArkKeys,
) -> Res<[u8; 32]> {
    loop {
        if let Message::Value(GameMessage::BoardDeclaration(mut proof, hash)) =
            net_receiver.get().await?
        {
            ui_sender.log_message(&format!(
                "Received board correctness proof from the other player. Hash {:?}.\nVerifying received proof...",
                hash
            ))?;

            let hash_clone = hash.to_vec();
            if spawn_thread_async(move || proof.is_correct(hash_clone.to_vec().into(), keys))
                .await??
            {
                ui_sender.log_message("Received proof is correct!")?;
                return Ok(hash);
            } else {
                ui_sender.log_message("Invalid proof")?;
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
            &mut game_context.ui_receiver,
            game_context.ui_sender.clone(),
            game_context.net_sender.clone(),
            game_context.keys.board_declaration_keys.clone(),
        ),
        receive_and_verify_board_proof(
            &mut game_context.net_receiver,
            &mut game_context.ui_sender,
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
