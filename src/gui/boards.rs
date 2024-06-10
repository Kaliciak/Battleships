use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;
use dioxus_desktop::*;

use crate::{
    gui::{ASSETS_DIR, GameScreenType, common::{BoardData, FieldState}},
    logic::GameState,
    model::{Direction, IncompleteBoard, Ship, SHIP_SIZES},
    ui::UiInput,
};

#[component]
pub fn Boards() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center",
            OpponentsBoard { style: "margin: 3em auto" }
            OurBoard { style: "margin: 3em auto" }
        }
    }
}

fn determine_opponents_board(state: Signal<Option<GameState>>) -> Vec<Vec<FieldState>> {
    let mut board_data = BoardData::new(vec![]);
    if state().is_none() {
        return board_data.board;
    }
    let state = state().expect("");
    board_data.add_shots(state.our_shots, FieldState::Ship);
    return board_data.board;
}

fn determine_our_board(state: Signal<Option<GameState>>) -> Vec<Vec<FieldState>> {
    if state().is_none() {
        let mut board_data = BoardData::new(vec![]);
        return board_data.board;
    }
    let state = state().expect("");
    let mut board_data = BoardData::new(state.board.board.ships.to_vec());
    board_data.add_shots(state.their_shots, FieldState::Hit);
    return board_data.board;
}

#[component]
fn OpponentsBoard(style: String) -> Element {
    let state = use_context::<Signal<Option<GameState>>>();
    let board = determine_opponents_board(state);

    rsx! {
        div {
            class: "board",
            style: "{style}",
            p { class: "column-labels-padding" }
            for i in 1..11 {
                p {
                    class: "column-label",
                    "{i}"
                }
            }
            for i in 1..11 {
                p {
                    class: "row-label",
                    "{i}"
                }
                for j in 1..11 {
                    button {
                        class: board[i][j].to_class_name(),
                        disabled: board[i][j] != FieldState::Empty,
                        onclick: move |_| {
                            let sender = use_context::<Sender<UiInput>>();
                            block_on(sender.send(UiInput::Shoot(j as u8, i as u8))).expect("");
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn OurBoard(style: String) -> Element {
    let state = use_context::<Signal<Option<GameState>>>();
    let board = determine_our_board(state);

    rsx! {
        div {
            class: "board",
            style: "{style}",
            p { class: "column-labels-padding" }
            for i in 1..11 {
                p {
                    class: "column-label",
                    "{i}"
                }
            }
            for i in 1..11 {
                p {
                    class: "row-label",
                    "{i}"
                }
                for j in 1..11 {
                    button {
                        class: board[i][j].to_class_name(),
                        disabled: true,
                    }
                }
            }
        }
    }
}
