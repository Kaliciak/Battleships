use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;

use crate::{
    gui::{ASSETS_DIR, GameScreenType},
    model::{Direction, IncompleteBoard, Ship, SHIP_SIZES},
    ui::UiInput,
};

#[component]
pub fn Lobby() -> Element {
    use_context_provider(|| Signal::new(Direction::Horizontal));

    rsx! {
        div {
            style: "display: grid; grid-template-columns: 1fr 3fr; grid-template-rows: 1fr 1fr; grid-row-gap: 5em",
            RemainingShips {
                style: "grid-column: 1; grid-row: 1"
            }
            ShipDirection {
                style: "grid-column: 1; grid-row: 2"
            }
            Board {
                style: "grid-column:2; grid-row: 1/3"
            }
        }
    }
}

#[derive(Clone)]
struct State {
    ships: Vec<u8>,
    board: Vec<Vec<u8>>,
    current_ship_size: u8,
}

fn determine_state(inc_board: Signal<IncompleteBoard>) -> State {
    let mut ships = vec![];
    for s in SHIP_SIZES {
        while ((s+1) as usize) > ships.len() {
            ships.push(0);
        }
        ships[s as usize] += 1;
    }

    let mut board = vec![ vec![0] ];
    for _ in 0..11 {
        let val = board[0][0].clone();
        board[0].push(val);
    }
    for _ in 0..11 {
        let val = board[0].clone();
        board.push(val);
    }

    for ship in inc_board().0 {
        ships[(ship.size as usize)] -= 1;
        if ship.direction == Direction::Horizontal {
            for i in 0..ship.size {
                board[ship.y as usize][(ship.x + i) as usize] = 2;
                board[(ship.y - 1) as usize][(ship.x + i) as usize] = 1;
                board[(ship.y + 1) as usize][(ship.x + i) as usize] = 1;
            }
            board[(ship.y - 1) as usize][(ship.x - 1) as usize] = 1;
            board[(ship.y - 1) as usize][(ship.x + ship.size) as usize] = 1;
            board[ship.y as usize][(ship.x - 1) as usize] = 1;
            board[ship.y as usize][(ship.x + ship.size) as usize] = 1;
            board[(ship.y + 1) as usize][(ship.x - 1) as usize] = 1;
            board[(ship.y + 1) as usize][(ship.x + ship.size) as usize] = 1;
        } else {
            for i in 0..ship.size {
                board[(ship.y + i) as usize][ship.x as usize] = 2;
                board[(ship.y + i) as usize][(ship.x - 1) as usize] = 1;
                board[(ship.y + i) as usize][(ship.x + 1) as usize] = 1;
            }
            board[(ship.y - 1) as usize][(ship.x - 1) as usize] = 1;
            board[(ship.y - 1) as usize][ship.x as usize] = 1;
            board[(ship.y - 1) as usize][(ship.x + 1) as usize] = 1;
            board[(ship.y + ship.size) as usize][(ship.x - 1) as usize] = 1;
            board[(ship.y + ship.size) as usize][ship.x as usize] = 1;
            board[(ship.y + ship.size) as usize][(ship.x + 1) as usize] = 1;
        }
    }

    let mut current_ship_size: u8 = 0;
    for i in 0..ships.len() {
        if ships[i] > 0 {
            current_ship_size = i as u8;
            break;
        }
    }

    State {
        ships,
        board,
        current_ship_size
    }
}

#[component]
fn RemainingShips(style: String) -> Element {
    let inc_board = use_context::<Signal<IncompleteBoard>>();
    let state = determine_state(inc_board);
    let ships = state.ships;

    rsx! {
        div {
            style: "{style}; display: grid; grid-template-columns: auto 2em 2em 2em 2em 2em 4em auto; gap: 0.3em",
            div {
                style: "grid-column: 1; grid-row: 2/7; margin: 0 auto"
            }
            div {
                style: "grid-column: 8; grid-row: 2/7; margin: 0 auto"
            }
            h2 {
                style: "grid-column: 1/9; font-size: 2em",
                "Remaining ships:"
            }
            for i in 0..ships.len() {
                if ships[i] > 0 {
                    for j in 2..i+2 {
                        div {
                            grid_column: "{j}",
                            display: "inline",
                            width: "2em",
                            height: "2em",
                            margin: "0",
                            background: "black",
                            border: "none",
                            outline: "none"
                        }
                    }
                    p {
                        style: "grid-column: 7; font-size: 2em; margin: 0.3em",
                        "x {ships[i]}"
                    }
                }
            }
        }
    }
}

#[component]
fn ShipDirection(style: String) -> Element {
    let mut direction = use_context::<Signal<Direction>>();

    rsx! {
        fieldset {
            style: "{style}; border: none; display: grid; grid-template-columns: auto 2em 12em auto; grid-template-rows: 2em 2em 2em auto; grid-row-gap: 0.3em",
            div {
                style: "grid-column: 1; grid-row: 2/4; margin: 0 auto"
            }
            div {
                style: "grid-column: 4; grid-row: 2/4; margin: 0 auto"
            }
            div {
                style: "grid-column: 1/5; grid-row: 4; margin: auto"
            }
            legend {
                style: "grid-column: 1/5; font-size: 2em",
                "Ship direction:"
            }
            input {
                style: "font-size: 2em; grid-column: 2",
                r#type: "radio",
                checked: if direction() == Direction::Horizontal { "true" } else { "false" },
                onclick: move |_| { direction.set(Direction::Horizontal) },
                id: "horizontal",
                tabindex: 0
            }
            label {
                style: "font-size: 2em; grid-column: 3; text-align: left",
                r#for: "horizontal",
                "horizontal"
            }
            input {
                style: "font-size: 2em; grid-column: 2",
                r#type: "radio",
                checked: if direction() == Direction::Vertical { "true" } else { "false" },
                onclick: move |_| { direction.set(Direction::Vertical) },
                id: "vertical",
                tabindex: 0
            }
            label {
                style: "font-size: 2em; grid-column: 3; text-align: left",
                r#for: "vertical",
                "vertical"
            }
        }
    }
}

#[component]
fn Board(style: String) -> Element {
    let direction = use_context::<Signal<Direction>>();
    let inc_board = use_context::<Signal<IncompleteBoard>>();
    let state = determine_state(inc_board);
    let board = state.board;

    rsx! {
        div {
            style: "{style}",
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
                                    block_on(sender.send(UiInput::PutShip(Ship { x: (j-2) as u8, y: (i-1) as u8, size: state.current_ship_size, direction: direction()}))).expect("");
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
