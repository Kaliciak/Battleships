use std::ops;

use crate::model::Ship;

#[derive(Debug, Clone, Copy)]
pub struct Point(pub i8, pub i8);
impl From<(i8, i8)> for Point {
    fn from(value: (i8, i8)) -> Self {
        Point(value.0, value.1)
    }
}
impl<T: Into<Point>> ops::Add<T> for Point {
    type Output = Point;

    fn add(self, rhs: T) -> Self::Output {
        let p: Point = rhs.into();
        Point(self.0 + p.0, self.1 + p.1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Interval(pub i8, pub i8);

impl Interval {
    pub fn contains(&self, x: i8) -> bool {
        self.0 <= x && x < self.1
    }
}

impl ops::BitXor<Interval> for Interval {
    type Output = bool;

    fn bitxor(self, rhs: Interval) -> Self::Output {
        self.contains(rhs.0) || rhs.contains(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle(pub Point, pub Point);

impl Rectangle {
    pub fn as_interval_pair(self) -> (Interval, Interval) {
        (
            Interval(self.0 .0, self.1 .0),
            Interval(self.0 .1, self.1 .1),
        )
    }
}

impl From<Ship> for Rectangle {
    fn from(value: Ship) -> Self {
        let (x, y) = value.direction.transpose(value.x, value.y, value.size - 1);
        Rectangle(
            Point(value.x as i8, value.y as i8),
            ((x + 1) as i8, (y + 1) as i8).into(),
        )
    }
}

impl From<Point> for Rectangle {
    fn from(value: Point) -> Self {
        Rectangle(value, value + (1, 1))
    }
}

impl<P: Into<Point>, Q: Into<Point>> From<(P, Q)> for Rectangle {
    fn from(value: (P, Q)) -> Self {
        Rectangle(value.0.into(), value.1.into())
    }
}

impl<P: Into<Rectangle>> ops::Add<P> for Rectangle {
    type Output = Rectangle;

    fn add(self, rhs: P) -> Self::Output {
        let Rectangle(a, b) = rhs.into();
        Rectangle(self.0 + a, self.1 + b)
    }
}

pub fn overlaps(a: impl Into<Rectangle>, b: impl Into<Rectangle>) -> bool {
    let (i1, i2) = a.into().as_interval_pair();
    let (j1, j2) = b.into().as_interval_pair();
    (i1 ^ j1) && (i2 ^ j2)
}
