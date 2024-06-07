use ark_std::iterable::Iterable;

use crate::{
    circuit::{
        board_declaration_circuit::CircuitField, field_declaration_circuit::FieldDeclarationCircuit,
    },
    crypto::proofs::{CorrectnessProof, PublicInput},
    gui::{GuiInput, GuiReceiver, GuiSender},
    logic::GameMessage,
    model::FieldState,
    net::message::Message,
    utils::{
        log::{Log, Logger},
        result::{Er, Res},
        threads::spawn_thread_async,
    },
};

use super::{
    board_creation::initialize_boards,
    main::{GameKeys, NetReceiver, NetSender},
    GameState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    Client,
    Host,
}
impl Player {
    pub fn other(&self) -> Self {
        match self {
            Player::Client => Player::Host,
            Player::Host => Player::Client,
        }
    }
}

/// All relevant information/channels available in the main game loop.
pub struct GameContext {
    pub player: Player,
    pub gui_receiver: GuiReceiver,
    pub gui_sender: GuiSender,
    pub net_receiver: NetReceiver,
    pub net_sender: NetSender,
    pub keys: GameKeys,
}

impl GameContext {
    pub async fn game_loop(&mut self) -> Res<()> {
        self.gui_sender.send(crate::gui::GuiMessage::Lobby).await?;
        let (board, their_hash) = initialize_boards(self).await?;
        self.gui_sender
            .log_message("Boards has been successfully initialized!")?;
        GameState {
            board,
            their_hash,
            our_role: self.player,
            our_shots: vec![],
            their_shots: vec![],
            turn_of: Player::Host,
        }
        .process(self)
        .await?;
        Ok(())
    }
}

impl GameState {
    async fn process(&mut self, game_context: &mut GameContext) -> Res<()> {
        loop {
            game_context
                .gui_sender
                .send(crate::gui::GuiMessage::PrintGameState(self.clone()))
                .await?;

            if are_all_discovered(&self.their_shots) {
                game_context.gui_sender.log_message("We have lost...")?;
                return Ok(());
            }
            if are_all_discovered(&self.our_shots) {
                game_context.gui_sender.log_message("We have won!")?;
                return Ok(());
            }

            if self.our_role == self.turn_of {
                game_context
                    .gui_sender
                    .log_message("Processing our turn...")?;
                self.process_our_turn(game_context).await?;
            } else {
                game_context
                    .gui_sender
                    .log_message("Processing turn of the other player...")?;
                self.process_their_turn(game_context).await?;
            }

            self.turn_of = self.turn_of.other();
        }
    }
    async fn process_our_turn(&mut self, game_context: &mut GameContext) -> Res<()> {
        loop {
            if let GuiInput::Shoot(x, y) = game_context.gui_receiver.get().await? {
                game_context
                    .gui_sender
                    .log_message(&format!("Shooting at ({x}, {y})"))?;
                game_context
                    .net_sender
                    .send(Message::Value(GameMessage::AskForField(x, y)))
                    .await?;
                loop {
                    if let Message::Value(GameMessage::FieldProof(mut proof, state)) =
                        game_context.net_receiver.get().await?
                    {
                        game_context
                            .gui_sender
                            .log_message("Received response, verifying...")?;
                        let keys_clone = game_context.keys.field_declaration_keys.clone();
                        let hash_input: PublicInput = self.their_hash.clone().to_vec().into();
                        if !spawn_thread_async(move || {
                            proof.is_correct(
                                hash_input
                                    + CircuitField::from(x)
                                    + CircuitField::from(y)
                                    + CircuitField::from(state as u8),
                                keys_clone,
                            )
                        })
                        .await??
                        {
                            return Err(Er {
                                message: "Invalid proof of the field!".to_owned(),
                            });
                        }
                        game_context.gui_sender.log_message(&format!(
                            "Received proof is correct. The field ({x}, {y}) is {}",
                            match state {
                                FieldState::Empty => "empty :(",
                                FieldState::Occupied => "occupied!",
                            }
                        ))?;
                        self.our_shots.push((x, y, state));
                        return Ok(());
                    }
                }
            }
        }
    }

    async fn process_their_turn(&mut self, game_context: &mut GameContext) -> Res<()> {
        game_context
            .gui_sender
            .log_message("Waiting for opponent's query")?;
        loop {
            if let Message::Value(GameMessage::AskForField(x, y)) =
                game_context.net_receiver.get().await?
            {
                game_context.gui_sender.log_message(&format!(
                    "Opponent asked for ({x}, {y}), generating proof..."
                ))?;
                let state = self.board.board.get_field_state(x, y);
                let circ: FieldDeclarationCircuit = (self.board, x, y).into();
                let logger: Logger = game_context.gui_sender.clone().into();
                let keys = game_context.keys.field_declaration_keys.clone();
                let proof =
                    spawn_thread_async(move || CorrectnessProof::create(circ, logger, keys))
                        .await??;

                game_context
                    .gui_sender
                    .log_message("Field proof generated, sending...")?;
                game_context
                    .net_sender
                    .send(Message::Value(GameMessage::FieldProof(proof, state)))
                    .await?;
                self.their_shots.push((x, y, state));
                return Ok(());
            }
        }
    }
}

fn are_all_discovered(shots: &Vec<(u8, u8, FieldState)>) -> bool {
    35 == shots.iter().map(|v| v.2).fold(0, |a, b| a + b as u8)
}
