#[derive(Copy, Clone, Debug)]
pub enum Direction {
    // downwards
    Vertical = 0,
    // to the right
    Horizontal = 1,
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
