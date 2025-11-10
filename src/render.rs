//! This module contains the logic for rendering the game.

use std::num::NonZeroU8;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, List, ListItem};

use crate::game::{Game, State};
use crate::playfield::Cell;

pub const SCALE: NonZeroU8 = NonZeroU8::new(2).unwrap();

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
        self.render_game(area, buf);
        // Render hold
        // Render next
        // Render score

        if self.state == State::GameOver {
            self.render_game_over(area, buf);
        }
    }
}

impl Game {
    fn render_game_over(&self, area: Rect, buf: &mut Buffer) {
        let items = ["Restart", "Exit"];

        let [layout] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Length(20)])
            .areas(area);

        let [inner] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Length(
                u16::try_from(items.len() + 2).unwrap(),
            )])
            .areas(layout);

        let title = Line::from(" Game Over ".bold());
        let popup = Block::bordered()
            .title(title.centered())
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::default()));

        let list_items = items
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let style = if i == usize::from(self.selected as u8) {
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::default())
                };

                ListItem::new(Line::from(*text).style(style))
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(list_items)
            .block(popup)
            .style(Style::default().bg(Color::default()));

        Widget::render(list, inner, buf);
    }

    fn render_game(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Tetris ".bold());

        let game_block = Block::bordered()
            .title(title.centered())
            .border_set(border::DOUBLE);

        (&game_block).render(area, buf);

        let inner = game_block.inner(area);

        self.render_board(&inner, buf);
        self.render_active_tetrimino(&inner, buf);
    }

    fn render_board(&self, area: &Rect, buf: &mut Buffer) {
        for (dy, row) in self.playfield.cells.into_iter().enumerate() {
            for (dx, column) in row.into_iter().enumerate() {
                let color = match column {
                    Cell::Empty => continue,
                    Cell::Occupied(color) => color,
                };

                let scale = i32::from(SCALE.get());

                let x: i32 = i32::from(area.x)
                    + (i32::try_from(dx).unwrap() * i32::from(TETRIMINO_WIDTH) * scale);
                let y: i32 = i32::from(area.y)
                    + (i32::try_from(dy).unwrap() * i32::from(TETRIMINO_HEIGHT) * scale);

                self.render_tetrimino(buf, x, y, color);
            }
        }
    }

    fn render_active_tetrimino(&self, area: &Rect, buf: &mut Buffer) {
        for (dy, row) in self.active.tetrimino.cells().into_iter().enumerate() {
            for (dx, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                let scale = i32::from(SCALE.get());

                // X/Y offsets will never be greater than the length/height of a tetrimino.
                assert_eq!(self.active.tetrimino.cells().len(), 4);
                let dx = i32::try_from(dx).unwrap();
                assert_eq!(row.len(), 4);
                let dy = i32::try_from(dy).unwrap();

                let offset_x = dx * i32::from(TETRIMINO_WIDTH) * scale;
                let width = self.active.x * i32::from(TETRIMINO_WIDTH) * scale;
                let x = i32::from(area.x) + offset_x + width;

                let offset_y = dy * i32::from(TETRIMINO_HEIGHT) * scale;
                let height = self.active.y * i32::from(TETRIMINO_HEIGHT) * scale;
                let y = i32::from(area.y) + offset_y + height;

                self.render_tetrimino(buf, x, y, self.active.tetrimino.color());
            }
        }
    }

    /// Draw a Tetrimino on the screen.
    fn render_tetrimino(&self, buf: &mut Buffer, x: i32, y: i32, color: Color) {
        let scale = u16::from(SCALE.get());

        for h in 0..TETRIMINO_HEIGHT * scale {
            for w in 0..TETRIMINO_WIDTH * scale {
                let x = u16::try_from(x + i32::from(w)).unwrap();
                let y = u16::try_from(y + i32::from(h)).unwrap();

                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(Style::default().bg(color));
                }
            }
        }
    }
}
