use std::fmt::{self, Display, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum CompassDirection {
    North,
    South,
    West,
    East,
}

impl CompassDirection {
    pub fn reversed(&self) -> CompassDirection {
        use CompassDirection::*;
        match self {
            North => South,
            South => North,
            East => West,
            West => East,
        }
    }
}

impl From<CompassDirection> for char {
    fn from(d: CompassDirection) -> char {
        use CompassDirection::*;
        match d {
            North => 'N',
            East => 'E',
            South => 'S',
            West => 'W',
        }
    }
}

pub const ALL_MOVE_OPTIONS: [CompassDirection; 4] = [
    CompassDirection::North,
    CompassDirection::East,
    CompassDirection::South,
    CompassDirection::West,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl Position {
    pub fn move_direction(&self, d: &CompassDirection) -> Position {
        match d {
            CompassDirection::North => Position {
                y: self.y - 1,
                ..*self
            },
            CompassDirection::South => Position {
                y: self.y + 1,
                ..*self
            },
            CompassDirection::East => Position {
                x: self.x + 1,
                ..*self
            },
            CompassDirection::West => Position {
                x: self.x - 1,
                ..*self
            },
        }
    }
}

pub fn bounds<'a, I>(points: I) -> Option<(Position, Position)>
where
    I: IntoIterator<Item = &'a Position>,
{
    let mut min_x: Option<i64> = None;
    let mut max_x: Option<i64> = None;
    let mut min_y: Option<i64> = None;
    let mut max_y: Option<i64> = None;
    fn maybe_update_min(min: &mut Option<i64>, val: i64) {
        match min {
            None => {
                *min = Some(val);
            }
            Some(v) if *v > val => *min = Some(val),
            Some(_) => (),
        }
    }
    fn maybe_update_max(max: &mut Option<i64>, val: i64) {
        match max {
            None => {
                *max = Some(val);
            }
            Some(v) if *v < val => *max = Some(val),
            Some(_) => (),
        }
    }
    for p in points.into_iter() {
        maybe_update_min(&mut min_x, p.x);
        maybe_update_max(&mut max_x, p.x);
        maybe_update_min(&mut min_y, p.y);
        maybe_update_max(&mut max_y, p.y);
    }
    match (min_x, max_x, min_y, max_y) {
        (Some(xlow), Some(xhigh), Some(ylow), Some(yhigh)) => {
            let min: Position = Position { x: xlow, y: ylow };
            let max: Position = Position { x: xhigh, y: yhigh };
            Some((min, max))
        }
        _ => None,
    }
}
