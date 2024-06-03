use crate::utils::ship_helpers::*;

pub const SHIP_SIZES: [u8; 15] = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4, 5];

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    // downwards
    VERTICAL = 0,
    // to the right
    HORIZONTAL = 1,
}

impl Direction {
    pub fn transpose(&self, x: u8, y: u8, size: u8) -> (u8, u8) {
        match self {
            Direction::VERTICAL => (x, y + size),
            Direction::HORIZONTAL => (x + size, y),
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

#[derive(Clone, Debug)]
pub struct IncompleteBoard(pub Vec<Ship>);

impl IncompleteBoard {
    pub fn new() -> Self {
        IncompleteBoard(vec![])
    }

    pub fn can_be_extended_with(&self, ship: Ship) -> bool {
        if !in_board((ship.x, ship.y)) {
            return false;
        }
        if !in_board(ship.direction.transpose(ship.x, ship.y, ship.size - 1)) {
            return false;
        }

        if !ship.size == SHIP_SIZES[self.0.len()] {
            return false;
        }

        if self.0.iter().any(|s| ships_collide(*s, ship)) {
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
