use dioxus::prelude::*;

use crate::{
    model::{self, Direction, Ship},
    ui::gui::ASSETS_DIR,
};

#[component]
pub fn ControlPanelStyle(style: String, children: Element) -> Element {
    rsx! {
        div {
            class: "control-panel",
            style: "{style}",

            div {
                style: "position: relative; height: 36px",
                img {
                    style: "position: absolute; top: 5px; left: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
                img {
                    style: "position: absolute; top: 5px; right: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
            }

            div {
                style: "padding: 0 36px",
                {children}
            }

            div {
                style: "position: relative; height: 36px",
                img {
                    style: "position: absolute; bottom: 5px; left: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
                img {
                    style: "position: absolute; bottom: 5px; right: 5px",
                    src: "{ASSETS_DIR}/screw.svg"
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum FieldState {
    Empty,
    Miss,
    Ship,
    Hit,
}

impl FieldState {
    pub fn to_class_name(&self) -> String {
        match self {
            FieldState::Empty => "field-state-empty",
            FieldState::Miss => "field-state-miss",
            FieldState::Ship => "field-state-ship",
            FieldState::Hit => "field-state-hit",
        }
        .to_string()
    }
}

pub struct BoardData {
    pub board: Vec<Vec<FieldState>>,
}

impl BoardData {
    pub fn new(ships: Vec<Ship>) -> Self {
        let mut board = vec![vec![FieldState::Empty]];
        for _ in 0..11 {
            let val = board[0][0].clone();
            board[0].push(val);
        }
        for _ in 0..11 {
            let val = board[0].clone();
            board.push(val);
        }

        for ship in ships {
            if ship.direction == Direction::Horizontal {
                for i in 0..ship.size {
                    board[ship.y as usize][(ship.x + i) as usize] = FieldState::Ship;
                }
            } else {
                for i in 0..ship.size {
                    board[(ship.y + i) as usize][ship.x as usize] = FieldState::Ship;
                }
            }
        }

        BoardData { board }
    }

    pub fn add_borders(&mut self, ships: Vec<Ship>) {
        for ship in ships {
            if ship.direction == Direction::Horizontal {
                for i in 0..ship.size {
                    self.board[(ship.y - 1) as usize][(ship.x + i) as usize] = FieldState::Miss;
                    self.board[(ship.y + 1) as usize][(ship.x + i) as usize] = FieldState::Miss;
                }
                self.board[(ship.y - 1) as usize][(ship.x - 1) as usize] = FieldState::Miss;
                self.board[(ship.y - 1) as usize][(ship.x + ship.size) as usize] = FieldState::Miss;
                self.board[ship.y as usize][(ship.x - 1) as usize] = FieldState::Miss;
                self.board[ship.y as usize][(ship.x + ship.size) as usize] = FieldState::Miss;
                self.board[(ship.y + 1) as usize][(ship.x - 1) as usize] = FieldState::Miss;
                self.board[(ship.y + 1) as usize][(ship.x + ship.size) as usize] = FieldState::Miss;
            } else {
                for i in 0..ship.size {
                    self.board[(ship.y + i) as usize][(ship.x - 1) as usize] = FieldState::Miss;
                    self.board[(ship.y + i) as usize][(ship.x + 1) as usize] = FieldState::Miss;
                }
                self.board[(ship.y - 1) as usize][(ship.x - 1) as usize] = FieldState::Miss;
                self.board[(ship.y - 1) as usize][ship.x as usize] = FieldState::Miss;
                self.board[(ship.y - 1) as usize][(ship.x + 1) as usize] = FieldState::Miss;
                self.board[(ship.y + ship.size) as usize][(ship.x - 1) as usize] = FieldState::Miss;
                self.board[(ship.y + ship.size) as usize][ship.x as usize] = FieldState::Miss;
                self.board[(ship.y + ship.size) as usize][(ship.x + 1) as usize] = FieldState::Miss;
            }
        }
    }

    pub fn add_shots(&mut self, shots: Vec<(u8, u8, model::FieldState)>, hit_state: FieldState) {
        for shot in shots {
            self.board[shot.1 as usize][shot.0 as usize] = if shot.2 == model::FieldState::Empty {
                FieldState::Miss
            } else {
                hit_state.clone()
            };
        }
    }
}
