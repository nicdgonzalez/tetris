use rand::seq::IndexedRandom as _;
use ratatui::style::Color;

pub type Cells = [[u16; 4]; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hitbox {
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
    pub right: u16,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Shape {
    pub fn random() -> Self {
        let variants = [
            Self::I,
            Self::O,
            Self::T,
            Self::S,
            Self::Z,
            Self::J,
            Self::L,
        ];
        // Tetris has an algorithm for properly generating tetriminoes:
        // https://tetris.fandom.com/wiki/Random_Generator
        //
        // (This is not that algorithm, and it ocassionally shows.)
        *variants.choose(&mut rand::rng()).unwrap()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    #[default]
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateDirection {
    Clockwise,
    Counterclockwise,
}

impl Orientation {
    pub fn rotate(self, direction: RotateDirection) -> Self {
        match (direction, self) {
            (RotateDirection::Clockwise, Self::Up) => Self::Right,
            (RotateDirection::Clockwise, Self::Right) => Self::Down,
            (RotateDirection::Clockwise, Self::Down) => Self::Left,
            (RotateDirection::Clockwise, Self::Left) => Self::Up,
            (RotateDirection::Counterclockwise, Self::Up) => Self::Left,
            (RotateDirection::Counterclockwise, Self::Left) => Self::Down,
            (RotateDirection::Counterclockwise, Self::Down) => Self::Right,
            (RotateDirection::Counterclockwise, Self::Right) => Self::Up,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tetrimino {
    pub shape: Shape,
    pub orientation: Orientation,
}

impl Tetrimino {
    pub const fn color(self) -> Color {
        match self.shape {
            Shape::I => Color::Rgb(49, 199, 239),
            Shape::O => Color::Rgb(247, 211, 8),
            Shape::T => Color::Rgb(173, 77, 156),
            Shape::S => Color::Rgb(66, 182, 66),
            Shape::Z => Color::Rgb(239, 32, 41),
            Shape::J => Color::Rgb(90, 101, 173),
            Shape::L => Color::Rgb(239, 121, 33),
        }
    }

    pub fn cells(self) -> Cells {
        // Rust requires `usize` to occupy be at least 16 bits, so casting from 8 bits is OK.
        // Related: https://github.com/rust-lang/rust/issues/48593
        let idx = usize::from(self.orientation as u8);

        match self.shape {
            Shape::I => BLOCK_I[idx],
            Shape::O => BLOCK_O,
            Shape::T => BLOCK_T[idx],
            Shape::S => BLOCK_S[idx],
            Shape::Z => BLOCK_Z[idx],
            Shape::J => BLOCK_J[idx],
            Shape::L => BLOCK_L[idx],
        }
    }

    pub fn hitbox(self) -> Hitbox {
        // Rust requires `usize` to occupy be at least 16 bits, so casting from 8 bits is OK.
        // Related: https://github.com/rust-lang/rust/issues/48593
        let idx = usize::from(self.orientation as u8);

        match self.shape {
            Shape::I => HITBOX_I[idx],
            Shape::O => HITBOX_O,
            Shape::T => HITBOX_T[idx],
            Shape::S => HITBOX_S[idx],
            Shape::Z => HITBOX_Z[idx],
            Shape::J => HITBOX_J[idx],
            Shape::L => HITBOX_L[idx],
        }
    }

    pub fn rotate(&mut self, direction: RotateDirection) {
        self.orientation = self.orientation.rotate(direction);
    }
}

#[rustfmt::skip]
const BLOCK_I: [Cells; 4] = [
    [
        [0, 0, 0, 0],
        [1, 1, 1, 1],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
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
];

const HITBOX_I: [Hitbox; 4] = [
    Hitbox {
        top: 1,
        left: 0,
        bottom: 1,
        right: 3,
    },
    Hitbox {
        top: 0,
        left: 2,
        bottom: 3,
        right: 2,
    },
    Hitbox {
        top: 2,
        left: 0,
        bottom: 2,
        right: 3,
    },
    Hitbox {
        top: 0,
        left: 1,
        bottom: 3,
        right: 1,
    },
];

#[rustfmt::skip]
const BLOCK_O: Cells = [
    [0, 1, 1, 0],
    [0, 1, 1, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

const HITBOX_O: Hitbox = Hitbox {
    top: 0,
    left: 1,
    bottom: 1,
    right: 2,
};

#[rustfmt::skip]
const BLOCK_T: [Cells; 4] = [
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

const HITBOX_T: [Hitbox; 4] = [
    Hitbox {
        top: 0,
        left: 0,
        bottom: 1,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 1,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 0,
        bottom: 2,
        right: 1,
    },
];

#[rustfmt::skip]
const BLOCK_S: [Cells; 4] = [
    [
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [0, 1, 1, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_S: [Hitbox; 4] = [
    Hitbox {
        top: 0,
        left: 0,
        bottom: 1,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 1,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 0,
        bottom: 2,
        right: 1,
    },
];

#[rustfmt::skip]
const BLOCK_Z: [Cells; 4] = [
    [
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 1, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_Z: [Hitbox; 4] = [
    Hitbox {
        top: 0,
        left: 0,
        bottom: 1,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 1,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 0,
        bottom: 2,
        right: 1,
    },
];

#[rustfmt::skip]
const BLOCK_J: [Cells; 4] = [
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
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_J: [Hitbox; 4] = [
    Hitbox {
        top: 0,
        left: 0,
        bottom: 1,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 1,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 0,
        bottom: 2,
        right: 1,
    },
];

#[rustfmt::skip]
const BLOCK_L: [Cells; 4] = [
    [
        [0, 0, 1, 0],
        [1, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
    ],
    [
        [0, 0, 0, 0],
        [1, 1, 1, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ],
];

const HITBOX_L: [Hitbox; 4] = [
    Hitbox {
        top: 0,
        left: 0,
        bottom: 1,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 1,
        left: 0,
        bottom: 2,
        right: 2,
    },
    Hitbox {
        top: 0,
        left: 0,
        bottom: 2,
        right: 1,
    },
];
