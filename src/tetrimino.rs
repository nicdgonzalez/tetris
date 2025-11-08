use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub column: i16,
    pub row: i16,
}

impl Point {
    pub const fn new(row: i16, column: i16) -> Self {
        Self { column, row }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Tetrimino {
    pub shape: Shape,
    pub orientation: Orientation,
}

impl Tetrimino {
    pub fn cells(&self) -> [[u8; 4]; 4] {
        self.shape.cells(self.orientation)
    }

    pub fn color(&self) -> Color {
        self.shape.color()
    }

    pub fn hitbox_top_left(&self) -> Point {
        let index = usize::from(self.orientation as u8);

        match self.shape {
            Shape::I => HITBOX_I[index].0,
            Shape::O => HITBOX_O.0,
            Shape::T => HITBOX_T[index].0,
            Shape::S => HITBOX_S[index].0,
            Shape::Z => HITBOX_Z[index].0,
            Shape::J => HITBOX_J[index].0,
            Shape::L => HITBOX_L[index].0,
        }
    }

    pub fn hitbox_bottom_right(self) -> Point {
        let index = usize::from(self.orientation as u8);

        match self.shape {
            Shape::I => HITBOX_I[index].1,
            Shape::O => HITBOX_O.1,
            Shape::T => HITBOX_T[index].1,
            Shape::S => HITBOX_S[index].1,
            Shape::Z => HITBOX_Z[index].1,
            Shape::J => HITBOX_J[index].1,
            Shape::L => HITBOX_L[index].1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Shape {
    #[default]
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orientation {
    #[default]
    Up,
    Left,
    Down,
    Right,
}

impl Shape {
    pub fn cells(self, orientation: Orientation) -> [[u8; 4]; 4] {
        let index = usize::from(orientation as u8);

        match self {
            Self::I => BLOCK_I[index],
            Self::O => BLOCK_O,
            Self::T => BLOCK_T[index],
            Self::S => BLOCK_S[index],
            Self::Z => BLOCK_Z[index],
            Self::J => BLOCK_J[index],
            Self::L => BLOCK_L[index],
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::I => Color::LightBlue,
            Self::O => Color::Yellow,
            Self::T => Color::Magenta,
            Self::S => Color::Green,
            Self::Z => Color::Red,
            Self::J => Color::Blue,
            Self::L => Color::Cyan,
        }
    }
}

#[rustfmt::skip]
const BLOCK_I: [[[u8; 4]; 4]; 4] = [
    [
        [0, 0, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 1, 0],
    ],
    [
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [1, 1, 1, 1],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 1],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_I: [(Point, Point); 4] = [
    (Point::new(0, 2), Point::new(3, 2)),
    (Point::new(2, 0), Point::new(2, 3)),
    (Point::new(0, 1), Point::new(3, 1)),
    (Point::new(1, 0), Point::new(1, 3)),
];

#[rustfmt::skip]
const BLOCK_O: [[u8; 4]; 4] = [
    [0, 1, 1, 0],
    [0, 1, 1, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

const HITBOX_O: (Point, Point) = (Point::new(0, 1), Point::new(1, 2));

#[rustfmt::skip]
const BLOCK_T: [[[u8; 4]; 4]; 4] = [
    [
        [0, 1, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_T: [(Point, Point); 4] = [
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 1), Point::new(2, 2)),
    (Point::new(1, 0), Point::new(2, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
];

#[rustfmt::skip]
const BLOCK_S: [[[u8; 4]; 4]; 4] = [
    [
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_S: [(Point, Point); 4] = [
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
];

#[rustfmt::skip]
const BLOCK_Z: [[[u8; 4]; 4]; 4] = [
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_Z: [(Point, Point); 4] = [
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
];

#[rustfmt::skip]
const BLOCK_J: [[[u8; 4]; 4]; 4] = [
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_J: [(Point, Point); 4] = [
    (Point::new(0, 0), Point::new(2, 1)),
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 1), Point::new(2, 2)),
    (Point::new(1, 0), Point::new(2, 2)),
];

#[rustfmt::skip]
const BLOCK_L: [[[u8; 4]; 4]; 4] = [
    [
        [1, 0, 0, 0],
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 1, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 1, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_L: [(Point, Point); 4] = [
    (Point::new(0, 0), Point::new(2, 1)),
    (Point::new(0, 0), Point::new(1, 2)),
    (Point::new(0, 0), Point::new(2, 1)),
    (Point::new(0, 0), Point::new(1, 2)),
];
