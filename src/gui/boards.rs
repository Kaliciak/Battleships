use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;
use dioxus_desktop::*;

use crate::{
    gui::{ASSETS_DIR, GameScreenType},
    logic::GameState,
    model::{Direction, FieldState, IncompleteBoard, Ship, SHIP_SIZES},
    ui::UiInput,
};

#[component]
pub fn Boards() -> Element {
    rsx! {
        div {
            style: "display: grid; grid-template-columns: 1fr 1fr",
            OpponentsBoard { }
            OurBoard { }
        }
    }
}

fn determine_opponents_board(state: Signal<Option<GameState>>) -> Vec<Vec<u8>> {
    let mut board = vec![ vec![0] ];
    for _ in 0..11 {
        let val = board[0][0].clone();
        board[0].push(val);
    }
    for _ in 0..11 {
        let val = board[0].clone();
        board.push(val);
    }

    if state().is_some() {
        for shot in state().expect("").our_shots {
            board[shot.1 as usize][shot.0 as usize] = if shot.2 == FieldState::Empty { 1 } else { 2 };
        }
    }

    return board;
}

fn determine_our_board(state: Signal<Option<GameState>>) -> Vec<Vec<u8>> {
    let mut board = vec![ vec![0] ];
    for _ in 0..11 {
        let val = board[0][0].clone();
        board[0].push(val);
    }
    for _ in 0..11 {
        let val = board[0].clone();
        board.push(val);
    }

    if state().is_some() {
        for ship in state().expect("").board.board.ships {
            if ship.direction == Direction::Horizontal {
                for i in 0..ship.size {
                    board[ship.y as usize][(ship.x + i) as usize] = 2;
                }
            } else {
                for i in 0..ship.size {
                    board[(ship.y + i) as usize][ship.x as usize] = 2;
                }
            }
        }

        for shot in state().expect("").their_shots {
            board[shot.1 as usize][shot.0 as usize] = if shot.2 == FieldState::Empty { 1 } else { 3 };
        }
    }
    
    return board;
}

#[component]
fn OpponentsBoard() -> Element {
    let state = use_context::<Signal<Option<GameState>>>();
    let board = determine_opponents_board(state);

    rsx! {
        div {
            h2 {
                style: "margin: 0 auto; font-size: 2em",
                "Opponent's board"
            }
            div {
                div {
                    style: "display: grid; grid-template-columns: auto repeat(11, 5em) auto; gap: 0.5em",
                    p {
                        style: "margin: 0 auto; padding: 0; grid-column: 1/3; grid-row: 1",
                        ""
                    }
                    for i in 2..12 {
                        p {
                            style: "margin: 0; padding: 0; font-size: 3em; grid-colum: {i}; grid-row: 1",
                            "{i-1}"
                        }
                    }
                    p {
                        style: "margin: 0 auto; padding: 0; grid-column: 13; grid-row: 1",
                        ""
                    }
                    for i in 2..12 {
                        p {
                            style: "margin: 0 auto; padding: 0; grid-column: 1; grid-row: {i}",
                            ""
                        }
                        p {
                            style: "margin: 0; padding: 0; font-size: 3em; grid-column: 2; grid-row: {i}; text-align: center",
                            "{i-1}"
                        }
                        for j in 3..13 {
                            if board[i-1][j-2] == 0 {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: white; grid-column: {j}; grid-row: {i}",
                                    onclick: move |_| {
                                        let sender = use_context::<Sender<UiInput>>();
                                        block_on(sender.send(UiInput::Shoot((j-2) as u8, (i-1) as u8))).expect("");
                                    }
                                }
                            } else if board[i-1][j-2] == 2 {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: black; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            } else {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: grey; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            }
                        }
                        p {
                            style: "margin: 0 auto; padding: 0; grid-column: 13; grid-row: {i}",
                            ""
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn OurBoard() -> Element {
    let state = use_context::<Signal<Option<GameState>>>();
    let board = determine_our_board(state);

    rsx! {
        div {
            h2 {
                style: "margin: 0 auto; font-size: 2em",
                "Our board"
            }
            div {
                div {
                    style: "display: grid; grid-template-columns: auto repeat(11, 5em) auto; gap: 0.5em",
                    p {
                        style: "margin: 0 auto; padding: 0; grid-column: 1/3; grid-row: 1",
                        ""
                    }
                    for i in 2..12 {
                        p {
                            style: "margin: 0; padding: 0; font-size: 3em; grid-colum: {i}; grid-row: 1",
                            "{i-1}"
                        }
                    }
                    p {
                        style: "margin: 0 auto; padding: 0; grid-column: 13; grid-row: 1",
                        ""
                    }
                    for i in 2..12 {
                        p {
                            style: "margin: 0 auto; padding: 0; grid-column: 1; grid-row: {i}",
                            ""
                        }
                        p {
                            style: "margin: 0; padding: 0; font-size: 3em; grid-column: 2; grid-row: {i}; text-align: center",
                            "{i-1}"
                        }
                        for j in 3..13 {
                            if board[i-1][j-2] == 0 {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: white; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            } else if board[i-1][j-2] == 2 {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: black; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            } else if board[i-1][j-2] == 1 {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: grey; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            } else {
                                button {
                                    style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: red; grid-column: {j}; grid-row: {i}",
                                    disabled: true
                                }
                            }
                        }
                        p {
                            style: "margin: 0 auto; padding: 0; grid-column: 13; grid-row: {i}",
                            ""
                        }
                    }
                }
            }
        }
    }
}
