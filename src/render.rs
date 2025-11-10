//! This module contains the logic for rendering the game.

use std::num::NonZeroU8;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, List, ListItem};

use crate::game::{Game, State};
use crate::playfield::{Cell, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};

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
        let [_hold, game, _next] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints(vec![
                Constraint::Length(20),
                // +2 because the walls count as part of the length.
                Constraint::Length(PLAYFIELD_WIDTH * TETRIMINO_WIDTH * u16::from(SCALE.get()) + 2),
                Constraint::Length(20),
            ])
            .areas(area);

        self.render_game(game, buf);
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
        // TODO: Figure out how to cleanly pad the items.
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
                let style = if i == usize::from(self.selected_option as u8) {
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
        let [game] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Length(
                // +2 because the ceiling and floor count as part of the length.
                PLAYFIELD_HEIGHT * TETRIMINO_HEIGHT * u16::from(SCALE.get()) + 2,
            )])
            .areas(area);

        let title = Line::from(" Tetris ".bold());

        let game_block = Block::bordered()
            .title(title.centered())
            .border_set(border::DOUBLE);

        (&game_block).render(game, buf);

        let inner = game_block.inner(game);

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

                for h in 0..TETRIMINO_HEIGHT * u16::from(SCALE.get()) {
                    for w in 0..TETRIMINO_WIDTH * u16::from(SCALE.get()) {
                        let x = u16::try_from(x + i32::from(w)).unwrap();
                        let y = u16::try_from(y + i32::from(h)).unwrap();

                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_style(Style::default().bg(color));
                        }
                    }
                }
            }
        }
    }

    fn render_active_tetrimino(&self, area: &Rect, buf: &mut Buffer) {
        for (dy, row) in self.active.tetrimino.cells().into_iter().enumerate() {
            for (dx, column) in row.into_iter().enumerate() {
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

                let scale = i32::from(SCALE.get());

                // Calculate our enlarged X to match the proper tetrimino width and scale.
                let width = self.active.x * i32::from(TETRIMINO_WIDTH) * scale;
                let offset_x = i32::try_from(dx).unwrap() * i32::from(TETRIMINO_WIDTH) * scale;
                let x = i32::from(area.x) + width + offset_x;

                // Calculate our enlarged Y to match the proper tetrimino height and scale.
                let height = self.active.y * i32::from(TETRIMINO_HEIGHT) * scale;
                let offset_y = i32::try_from(dy).unwrap() * i32::from(TETRIMINO_HEIGHT) * scale;
                let y = i32::from(area.y) + height + offset_y;

                for h in 0..TETRIMINO_HEIGHT * u16::from(SCALE.get()) {
                    for w in 0..TETRIMINO_WIDTH * u16::from(SCALE.get()) {
                        let x = u16::try_from(x + i32::from(w)).unwrap();
                        let y = u16::try_from(y + i32::from(h)).unwrap();

                        if let Some(cell) = buf.cell_mut((x, y)) {
                            cell.set_style(Style::default().bg(self.active.tetrimino.color()));
                        }
                    }
                }
            }
        }
    }
}
