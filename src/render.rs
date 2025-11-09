//! This module contains the logic for rendering widgets on the screen.

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

use crate::board::{Cell, BOARD_HEIGHT, BOARD_WIDTH};
use crate::game::Game;

const SCALE: u8 = 2;

// Terminal cells are 2x taller than they are wide, so using two cells makes a nice square block.
pub const TETRIMINO_WIDTH: u16 = 2;
pub const TETRIMINO_HEIGHT: u16 = 1;

// Top Left: Hold piece
// Top Right: Next piece
// Center: Game
// Bottom Right: Score/stats
// Bottom Left: Controls

impl Widget for &Game {
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
                Constraint::Length(BOARD_WIDTH * TETRIMINO_WIDTH * u16::from(SCALE) + 2),
                Constraint::Length(20),
            ])
            .areas(area);

        self.render_game(game, buf);
    }
}

impl Game {
    fn render_game(&self, area: Rect, buf: &mut Buffer) {
        let [game] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Length(
                // +2 because the ceiling and floor count as part of the length.
                BOARD_HEIGHT * TETRIMINO_HEIGHT * u16::from(SCALE) + 2,
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
        for (y, row) in self.playfield.cells.into_iter().enumerate() {
            for (x, column) in row.into_iter().enumerate() {
                let color = match column {
                    Cell::Empty => continue,
                    Cell::Occupied(color) => color,
                };

                let x = (area.x as i16) + (x as i16 * TETRIMINO_WIDTH as i16 * SCALE as i16);
                let y = area.y as i16 + (y as i16 * TETRIMINO_HEIGHT as i16 * SCALE as i16);

                for h in 0..TETRIMINO_HEIGHT * u16::from(SCALE) {
                    for w in 0..TETRIMINO_WIDTH * u16::from(SCALE) {
                        let x = x as u16 + w;
                        let y = y as u16 + h;

                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_symbol("█").set_style(Style::default().fg(color));
                        }
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

                let x: i32 = i32::from(area.x)
                    + (self.x * i32::from(TETRIMINO_WIDTH) * i32::from(SCALE))
                    + (i32::try_from(x).unwrap() * i32::from(TETRIMINO_WIDTH) * i32::from(SCALE));
                let y: i32 = i32::from(area.y)
                    + (self.y * i32::from(TETRIMINO_HEIGHT) * i32::from(SCALE))
                    + (i32::try_from(y).unwrap() * i32::from(TETRIMINO_HEIGHT) * i32::from(SCALE));

                for h in 0..TETRIMINO_HEIGHT * u16::from(SCALE) {
                    for w in 0..TETRIMINO_WIDTH * u16::from(SCALE) {
                        if let Some(cell) = buf.cell_mut((
                            u16::try_from(x + i32::from(w)).unwrap(),
                            u16::try_from(y + i32::from(h)).unwrap(),
                        )) {
                            cell.set_symbol("█")
                                .set_style(Style::default().fg(self.tetrimino.color()));
                        }
                    }
                }
            }
        }
    }
}
