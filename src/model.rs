pub enum Direction {
    VERTICAL, 
    HORIZONTAL,
}

pub struct Ship {
    pub x: UInt8,
    pub y: UInt8,
    pub size: UInt8,
    pub direction: Direction,
}

pub struct Board {
    pub ships: [Ship; 15],
}
