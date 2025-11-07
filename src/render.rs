//! This module contains the logic for rendering widgets on the screen.

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use crate::app::App;
use crate::board::{BOARD_HEIGHT, BOARD_WIDTH};

const SCALE: u16 = 2;

// Terminal cells are 2x taller than they are wide, so using two cells makes a nice square block.
pub const TETRIMINO_WIDTH: u16 = 2;
pub const TETRIMINO_HEIGHT: u16 = 1;

// Top Left: Hold piece
// Top Right: Next piece
// Center: Game
// Bottom Right: Score/stats
// Bottom Left: Controls

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [_hold, game, _next] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints(vec![
                Constraint::Length(20),
                // +2 because the walls count as part of the length.
                Constraint::Length(BOARD_WIDTH * TETRIMINO_WIDTH * SCALE + 2),
                Constraint::Length(20),
            ])
            .areas(area);

        self.render_game(game, buf);
    }
}

impl App {
    fn render_game(&self, area: Rect, buf: &mut Buffer) {
        let [game] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Length(
                // +2 because the ceiling and floor count as part of the length.
                BOARD_HEIGHT * TETRIMINO_HEIGHT * SCALE + 2,
            )])
            .areas(area);

        let title = Line::from(" Tetris ".bold());

        let game_block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        (&game_block).render(game, buf);

        let inner = game_block.inner(game);

        self.render_board(&inner, buf);
        self.render_active_tetrimino(&inner, buf);
    }

    fn render_board(&self, area: &Rect, buf: &mut Buffer) {
        for (y, row) in self.board.cells.into_iter().enumerate() {
            for (x, column) in row.into_iter().enumerate() {
                let x = area.x + (x as u16);
                let y = area.y + (y as u16);

                for w in 1..=TETRIMINO_WIDTH * SCALE {
                    if let Some(cell) = buf.cell_mut(Position::new(x + w, y + w)) {
                        if column.is_empty() {
                            continue;
                        }

                        cell.set_symbol("█")
                            .set_style(Style::default().bg(self.tetrimino.color()));
                    }
                }
            }
        }
    }

    fn render_active_tetrimino(&self, area: &Rect, buf: &mut Buffer) {
        for (y, row) in self.tetrimino.cells().into_iter().enumerate() {
            for (x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                // Board literal:
                //
                // 0 1 0
                // 1 1 1
                //
                // Board render (without scaling): [w=2, h=1]
                //
                // 0 0 1 1 0 0
                // 1 1 1 1 1 1
                //
                // Board render (with scaling): [w=2, h=1]
                //
                // 0 0 0 0 1 1 1 1 0 0 0 0
                // 0 0 0 0 1 1 1 1 0 0 0 0
                // 1 1 1 1 1 1 1 1 1 1 1 1
                // 1 1 1 1 1 1 1 1 1 1 1 1

                let x = (area.x)
                    + (self.x * TETRIMINO_WIDTH * SCALE)
                    + (x as u16 * TETRIMINO_WIDTH * SCALE);
                // TODO: `y` is wrong here.
                // (area.y) + (self.y * TETRIMINO_HEIGHT * SCALE) + (y as u16 * TETRIMINO_HEIGHT);
                let y = area.y
                    + (self.y * TETRIMINO_HEIGHT * SCALE)
                    + (y as u16 * TETRIMINO_HEIGHT * SCALE);

                for h in 0..TETRIMINO_HEIGHT * SCALE {
                    for w in 0..TETRIMINO_WIDTH * SCALE {
                        if let Some(cell) = buf.cell_mut((x + w, y + h)) {
                            cell.set_symbol("█")
                                .set_style(Style::default().fg(self.tetrimino.color()));
                        }
                    }
                }
            }
        }
    }
}
