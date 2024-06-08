use async_channel::{Receiver, Sender};
use async_std::task::block_on;
use game_loop::Player;
use main::run_logic_async;
use serde::{Deserialize, Serialize};

use crate::{
    circuit::{
        board_declaration_circuit::BoardDeclarationCircuit,
        field_declaration_circuit::FieldDeclarationCircuit,
    },
    crypto::proofs::CorrectnessProof,
    gui::{GuiInput, GuiMessage},
    model::FieldState,
    utils::async_receiver::AsyncReceiver,
};

mod board_creation;
mod game_loop;
pub mod main;

/// Possible message received from another player
#[derive(Debug, Serialize, Deserialize)]
pub enum GameMessage {
    BoardDeclaration(CorrectnessProof<BoardDeclarationCircuit>, [u8; 32]),
    AskForField(u8, u8),
    FieldProof(CorrectnessProof<FieldDeclarationCircuit>, FieldState),
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: BoardDeclarationCircuit,
    pub their_hash: [u8; 32],
    pub our_role: Player,
    pub our_shots: Vec<(u8, u8, FieldState)>,
    pub their_shots: Vec<(u8, u8, FieldState)>,
    pub turn_of: Player,
}

pub fn run_logic_with_gui(gui_callback: impl Fn(Receiver<GuiMessage>, Sender<GuiInput>) -> ()) {
    let (s_input, r_input) = async_channel::unbounded::<GuiMessage>();
    let (s_output, r_output) = async_channel::unbounded::<GuiInput>();

    std::thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let _ = block_on(run_logic_async(AsyncReceiver(r_output), s_input));
        })
        .unwrap();

    gui_callback(r_input, s_output);
}
