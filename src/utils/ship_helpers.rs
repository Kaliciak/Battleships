use crate::Ship;

pub fn is_in_range(val: u8, begin_incl: u8, end_incl: u8) -> bool {
    begin_incl <= val && val <= end_incl
}

pub fn in_board(point: (u8, u8)) -> bool {
    is_in_range(point.0, 1, 10) && is_in_range(point.1, 1, 10)
}

pub fn ships_collide(ship_a: Ship, ship_b: Ship) -> bool {
    let (a_xmin, a_ymin) = (ship_a.x - 1, ship_a.y - 1);
    let (a_xmax, a_ymax) = ship_a
        .direction
        .transpose(ship_a.x + 1, ship_a.y + 1, ship_a.size - 1);
    (0..ship_b.size).any(|s| {
        let (b_x, b_y) = ship_b.direction.transpose(ship_b.x, ship_b.y, s);
        is_in_range(b_x, a_xmin, a_xmax) && is_in_range(b_y, a_ymin, a_ymax)
    })
}
