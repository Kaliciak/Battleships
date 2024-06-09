use async_channel::Sender;
use async_std::task::block_on;
use dioxus::prelude::*;

use crate::{
    gui::{ASSETS_DIR, GameScreenType},
    model::{Direction, IncompleteBoard, Ship},
    ui::UiInput,
};

#[component]
pub fn Lobby() -> Element {
    use_context_provider(|| Signal::new(Direction::Horizontal));
    rsx! {
        div {
            style: "display: flex",
            RemainingShips {}
            Board {}
        }
    }
}

#[component]
fn RemainingShips() -> Element {
    let mut ships = vec![5, 4, 3, 2, 1];
    let board = use_context::<Signal<IncompleteBoard>>();
    for ship in board().0 {
        ships[(ship.size-1) as usize] -= 1;
    }

    let mut direction = use_context::<Signal<Direction>>();

    rsx! {
        div {
            //style: "flex: 1; margin: 1em; display: block;",
            style: "flex: 1",
            h2 { "Remaining ships:" }
            for i in 0..ships.len() {
                if ships[i] > 0 {
                    div {
                        //style: "display: grid; grid-template-columns: auto auto auto auto auto auto; padding: auto; margin: auto",
                        style: "display: inline",
                        for j in 1..i+2 {
                            img {
                                //grid_column: "{j}",
                                display: "inline",
                                width: "20px",
                                height: "20px",
                                margin: "0",
                                background: "black",
                                border: "none",
                                outline: "none"
                            }
                        }
                        p {
                            //style: "grid-column: 6; font-size: 2em;",
                            style: "display: inline; font-size: 2em; margin: 0.3em",
                            "{ships[i]}"
                        }
                        br {}
                    }
                }
            }
            div { style: "height: 2em" }
            fieldset {
                style: "border: none",
                legend {
                    style: "font-size: 2em",
                    "Ship direction:"
                }
                input {
                    r#type: "radio",
                    checked: if direction() == Direction::Horizontal { "true" } else { "false" },
                    onclick: move |_| { direction.set(Direction::Horizontal) },
                    id: "horizontal",
                    tabindex: 0
                }
                label {
                    r#for: "horizontal",
                    "horizontal"
                }
                br {}
                input {
                    r#type: "radio",
                    checked: if direction() == Direction::Vertical { "true" } else { "false" },
                    onclick: move |_| { direction.set(Direction::Vertical) },
                    id: "vertical",
                    tabindex: 0
                }
                label {
                    r#for: "vertical",
                    "vertical"
                }
            }
        }
    }
}

#[component]
fn Board() -> Element {
    let direction = use_context::<Signal<Direction>>();
    rsx! {
        div {
            style: "flex: 3;",
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
                        button {
                            style: "padding: 0; margin: 0; font-size: 1em; width: 5em; height: 5em; background: white; grid-column: {j}; grid-row: {i}",
                            onclick: move |_| {
                                let sender = use_context::<Sender<UiInput>>();
                                block_on(sender.send(UiInput::PutShip(Ship { x: j-2, y: i-1, size: 1, direction: direction()}))).expect("");
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
