use ratatui::style::Color;

pub const PLAYFIELD_WIDTH: u16 = 10;
// TODO: Height is supposed to be 40 according to the Tetris Guideline, however, only 20 rows are
// supposed to be shown to the player. (Some implementations only use 22 rows.) I was already
// considering increasing the size to know when the game is properly over, but it requires creating
// a viewbox for the playfield that limits which portion is rendered, which I have not done yet.
pub const PLAYFIELD_HEIGHT: u16 = 20;

// Rust requires all pointer addresses to use at least 16 bits, so this should never panic.
//
// While I did do my due diligence and research prior, I am not an expert. If my assumptions are
// wrong, I want the program to fail loudly, hence the assertion.
const _: () = assert!(usize::BITS >= u16::BITS);
type Cells = [[Cell; PLAYFIELD_WIDTH as usize]; PLAYFIELD_HEIGHT as usize];

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
    /// Convenience method to check whether the cell is empty without requiring imports.
    pub fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }
}
