use ark_std::iterable::Iterable;
use serde::{Deserialize, Serialize};

use crate::utils::ship_helpers::*;

pub const SHIP_SIZES: [u8; 15] = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5];

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    // downwards
    Vertical = 0,
    // to the right
    Horizontal = 1,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FieldState {
    // No ship on the field
    Empty = 0,
    // There is a ship occupying the field
    Occupied = 1,
}

impl Direction {
    pub fn transpose(&self, x: u8, y: u8, size: u8) -> (u8, u8) {
        match self {
            Direction::Vertical => (x, y + size),
            Direction::Horizontal => (x + size, y),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Ship {
    pub x: u8,
    pub y: u8,
    pub size: u8,
    pub direction: Direction,
}

#[derive(Copy, Clone, Debug)]
pub struct Board {
    pub ships: [Ship; 15],
}

impl Board {
    pub fn get_field_state(&self, x: u8, y: u8) -> FieldState {
        if self
            .ships
            .iter()
            .any(|ship| overlaps(ship, Point(x as i8, y as i8)))
        {
            FieldState::Occupied
        } else {
            FieldState::Empty
        }
    }
}

#[derive(Clone, Debug)]
pub struct IncompleteBoard(pub Vec<Ship>);

impl IncompleteBoard {
    pub fn new() -> Self {
        IncompleteBoard(vec![])
    }

    pub fn can_be_extended_with(&self, ship: Ship) -> bool {
        let board_boundaries = ((1, 1), (11, 11));

        let ship_rect: Rectangle = ship.into();

        if !overlaps(board_boundaries, ship_rect.0)
            || !overlaps(board_boundaries, ship_rect.1 + (-1, -1))
        {
            return false;
        }
        if !ship.size == SHIP_SIZES[self.0.len()] {
            return false;
        }

        if self
            .0
            .iter()
            .any(|s| overlaps(*s, ship_rect + ((-1, -1), (1, 1))))
        {
            return false;
        }

        true
    }

    pub fn extend(&mut self, ship: Ship) {
        self.0.push(ship);
    }

    pub fn build_board(self) -> Board {
        Board {
            ships: self.0.try_into().unwrap(),
        }
    }
}

pub const SAMPLE_BOARD: Board = Board {
    ships: [
        Ship {
            x: 1,
            y: 1,
            size: 1,
            direction: Direction::Vertical,
        },
        Ship {
            x: 1,
            y: 3,
            size: 1,
            direction: Direction::Vertical,
        },
        Ship {
            x: 1,
            y: 5,
            size: 1,
            direction: Direction::Vertical,
        },
        Ship {
            x: 1,
            y: 7,
            size: 1,
            direction: Direction::Vertical,
        },
        Ship {
            x: 1,
            y: 9,
            size: 1,
            direction: Direction::Vertical,
        },
        Ship {
            x: 3,
            y: 1,
            size: 2,
            direction: Direction::Vertical,
        },
        Ship {
            x: 3,
            y: 4,
            size: 2,
            direction: Direction::Vertical,
        },
        Ship {
            x: 3,
            y: 7,
            size: 2,
            direction: Direction::Vertical,
        },
        Ship {
            x: 3,
            y: 10,
            size: 2,
            direction: Direction::Horizontal,
        },
        Ship {
            x: 5,
            y: 1,
            size: 3,
            direction: Direction::Vertical,
        },
        Ship {
            x: 5,
            y: 5,
            size: 3,
            direction: Direction::Vertical,
        },
        Ship {
            x: 6,
            y: 10,
            size: 3,
            direction: Direction::Horizontal,
        },
        Ship {
            x: 7,
            y: 1,
            size: 4,
            direction: Direction::Vertical,
        },
        Ship {
            x: 9,
            y: 1,
            size: 4,
            direction: Direction::Vertical,
        },
        Ship {
            x: 10,
            y: 6,
            size: 5,
            direction: Direction::Vertical,
        },
    ],
};
