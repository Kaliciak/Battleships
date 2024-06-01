#[derive(Copy, Clone, Debug)]
pub enum Direction {
    // downwards
    Vertical = 0,
    // to the right
    Horizontal = 1,
}

#[derive(Copy, Clone, Debug)]
pub enum FieldState {
    // No ship on the field
    Empty = 0,
    // There is a ship occupying the field
    Occupied = 1,
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
