use ratatui::style::Color;

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Debug, Clone, Copy, Default)]
pub struct Board {
    pub cells: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
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
