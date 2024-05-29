pub enum Direction {
    Vertical,
    Horizontal,
}

pub struct Ship {
    pub x: u8,
    pub y: u8,
    pub size: u8,
    pub direction: Direction,
}

pub struct Board {
    pub ships: [Ship; 15],
}
