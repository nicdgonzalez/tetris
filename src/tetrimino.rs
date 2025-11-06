use ratatui::style::Color;

// Terminal cells are taller than they are wide, so using two cells will make a nice square.
pub const TETRIMINO_WIDTH: usize = 2;
pub const TETRIMINO_HEIGHT: usize = 1;

#[derive(Debug, Clone, Copy, Default)]
pub struct Tetrimino {
    pub shape: Shape,
    pub orientation: Orientation,
}

impl Tetrimino {
    pub fn new(shape: Shape, orientation: Orientation) -> Self {
        Self { shape, orientation }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Shape {
    I,
    O,
    #[default]
    T,
    S,
    Z,
    J,
    L,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orientation {
    #[default]
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

impl Shape {
    pub fn get(self, orientation: Orientation) -> [[u8; 4]; 4] {
        match self {
            Self::I => BLOCK_I[orientation as usize],
            Self::O => BLOCK_O,
            Self::T => BLOCK_T[orientation as usize],
            Self::S => BLOCK_S[orientation as usize],
            Self::Z => BLOCK_Z[orientation as usize],
            Self::J => BLOCK_J[orientation as usize],
            Self::L => BLOCK_L[orientation as usize],
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
    ]
];

#[rustfmt::skip]
const BLOCK_O: [[u8; 4]; 4] = [
    [0, 1, 1, 0],
    [0, 1, 1, 0],
    [0, 0, 0, 0],
    [0, 0, 0, 0],
];

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
    ]
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
    ]
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
    ]
];

#[rustfmt::skip]
const BLOCK_J: [[[u8; 4]; 4]; 4] = [
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
    ]
];

#[rustfmt::skip]
const BLOCK_L: [[[u8; 4]; 4]; 4] = [
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
        [1, 1, 1, 0],
        [1, 0, 0, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ],
    [
        [1, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 1, 0, 0],
        [0, 0, 0, 0],
    ]
];
