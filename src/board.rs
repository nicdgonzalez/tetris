use ratatui::style::Color;

pub const BOARD_WIDTH: u16 = 10;
pub const BOARD_HEIGHT: u16 = 20;

// Rust requires all pointer addresses to use at least 16 bits, so this should never panic.
//
// While I did do my due diligence and research prior, but I'm not an expert. If I am wrong,
// I want the program to fail loudly.
const _: () = assert!(usize::BITS >= u16::BITS);
type Cells = [[Cell; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];

#[derive(Debug, Clone, Copy, Default)]
pub struct Playfield {
    pub cells: Cells,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Cell {
    #[default]
    Empty,
    Occupied(Color),
}

impl Cell {
    pub fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }
}
