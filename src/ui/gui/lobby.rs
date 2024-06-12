use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;

use crate::{
    model::{Direction, IncompleteBoard, Ship, SHIP_SIZES},
    ui::gui::common::{BoardData, ControlPanelStyle, FieldState},
    ui::UiInput,
};

#[component]
pub fn Lobby() -> Element {
    use_context_provider(|| Signal::new(Direction::Horizontal));

    rsx! {
        div {
            style: "display: flex; align-items: center",
            ControlPanelStyle {
                style: "width: auto; margin: 3em auto",
                div {
                    RemainingShips {
                        style: ""
                    }
                    ShipDirection {
                        style: ""
                    }
                }
            }
            Board {
                style: "margin: 3em auto",
            }
        }
    }
}

#[derive(Clone)]
struct State {
    ships: Vec<u8>,
    board: Vec<Vec<FieldState>>,
    current_ship_size: u8,
}

fn determine_state(inc_board: Signal<IncompleteBoard>) -> State {
    let mut ships = vec![];
    for s in SHIP_SIZES {
        while ((s + 1) as usize) > ships.len() {
            ships.push(0);
        }
        ships[s as usize] += 1;
    }

    for ship in inc_board().0 {
        ships[ship.size as usize] -= 1;
    }

    let mut board_data = BoardData::new(inc_board().0);
    board_data.add_borders(inc_board().0);

    let mut current_ship_size: u8 = 0;
    for (i, ship_count) in ships.iter().enumerate() {
        if *ship_count > 0 {
            current_ship_size = i as u8;
            break;
        }
    }

    State {
        ships,
        board: board_data.board,
        current_ship_size,
    }
}

#[component]
fn RemainingShips(style: String) -> Element {
    let inc_board = use_context::<Signal<IncompleteBoard>>();
    let state = determine_state(inc_board);
    let ships = state.ships;

    rsx! {
        div {
            style: "{style}; display: grid; grid-template-columns: 2em 2em 2em 2em 2em 4em; gap: 0.3em",
            h2 {
                style: "grid-column: 1/7; font-size: 2em",
                "Remaining ships:"
            }
            for i in 0..ships.len() {
                if ships[i] > 0 {
                    for j in 1..i+1 {
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
                        style: "grid-column: 6; font-size: 2em; margin: 0.3em",
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
            style: "{style}",
            h2 {
                class: "fieldset-title",
                "Ship direction:"
            }
            input {
                id: "horizontal",
                r#type: "radio",
                checked: if direction() == Direction::Horizontal { "true" } else { "false" },
                onclick: move |_| { direction.set(Direction::Horizontal) }
            }
            label {
                r#for: "horizontal",
                "horizontal"
            }
            input {
                id: "vertical",
                r#type: "radio",
                checked: if direction() == Direction::Vertical { "true" } else { "false" },
                onclick: move |_| { direction.set(Direction::Vertical) }
            }
            label {
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
            class: "board",
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
                            block_on(sender.send(UiInput::PutShip(
                                        Ship {
                                            x: j as u8,
                                            y: i as u8,
                                            size: state.current_ship_size,
                                            direction: direction()
                                        }))).expect("");
                        }
                    }
                }
            }
        }
    }
}
