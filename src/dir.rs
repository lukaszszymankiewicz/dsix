#[derive(Copy, Clone)]
pub enum Dir {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Dir {
    pub fn opposite(&self) -> Dir {
        match self {
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
        }
    }
}

