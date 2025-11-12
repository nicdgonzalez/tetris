//! This module contains the logic for rendering the game.

use std::num::NonZeroU8;

use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, List, ListItem, Padding, Paragraph};

use crate::app::{App, View};
use crate::game::GameState;
use crate::playfield::{Cell, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};

pub const SCALE: NonZeroU8 = NonZeroU8::new(2).unwrap();

// Terminal cells are 2x taller than they are wide, so using two cells makes a nice square block.
pub const TETRIMINO_WIDTH: u16 = 2;
pub const TETRIMINO_HEIGHT: u16 = 1;

const TETRIS: &str = "
████████╗███████╗████████╗██████╗ ██╗███████╗
╚══██╔══╝██╔════╝╚══██╔══╝██╔══██╗██║██╔════╝
   ██║   █████╗     ██║   ██████╔╝██║███████╗
   ██║   ██╔══╝     ██║   ██╔══██╗██║╚════██║
   ██║   ███████╗   ██║   ██║  ██║██║███████║
   ╚═╝   ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚══════╝
";

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.view {
            View::Menu => self.render_menu(area, buf),
            View::Game => self.render_game(area, buf),
        }
    }
}

impl App {
    fn render_menu(&self, area: Rect, buf: &mut Buffer) {
        let [title, text] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .areas(area);

        self.render_menu_title(title, buf);
        self.render_menu_text(text, buf);
    }

    fn render_menu_title(&self, area: Rect, buf: &mut Buffer) {
        let [_, layout] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::End)
            .constraints(vec![Constraint::Fill(0), Constraint::Length(7)])
            .areas(area);

        let title = Paragraph::new(TETRIS.lines().map(Line::from).collect::<Vec<_>>()).centered();

        title.render(layout, buf);
    }

    fn render_menu_text(&self, area: Rect, buf: &mut Buffer) {
        let line = Line::from("Press any key to continue").centered();
        line.render(area, buf);
    }

    fn render_game(&self, area: Rect, buf: &mut Buffer) {
        let scale = u16::from(SCALE.get());

        let playfield_height = PLAYFIELD_HEIGHT * TETRIMINO_HEIGHT * scale;
        let playfield_width = PLAYFIELD_WIDTH * TETRIMINO_WIDTH * scale;

        let [layout] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::Center)
            .constraints(vec![
                // Ceiling and floor count towards the length.
                Constraint::Length(playfield_height + 2),
            ])
            .areas(area);

        let [left, playfield, right] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints(vec![
                Constraint::Fill(1),
                // Both side walls count towards the length.
                Constraint::Length(playfield_width + 2),
                Constraint::Fill(1),
            ])
            .areas(layout);

        self.render_game_left(left, buf);
        self.render_game_playfield(playfield, buf);
        self.render_game_right(right, buf);

        if self.game.state == GameState::GameOver {
            self.render_game_game_over(area, buf);
        }
    }

    fn render_game_left(&self, area: Rect, buf: &mut Buffer) {
        let scale = u16::from(SCALE.get());

        let tetrimino_height = 4 * TETRIMINO_HEIGHT * scale;

        let [layout] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::End)
            .constraints(vec![Constraint::Length(25)])
            .areas(area);

        let [hold, controls] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::SpaceBetween)
            // +2 for top/bottom border.
            .constraints(vec![
                Constraint::Length(tetrimino_height + 2),
                Constraint::Length(7 + 2 + 2),
            ])
            .areas(layout);

        self.render_game_left_hold(hold, buf);
        self.render_game_left_controls(controls, buf);
    }

    fn render_game_left_hold(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Hold "))
            .border_set(border::DOUBLE);

        // Render held tetrimino.
        if let Some(tetrimino) = self.game.hold {
            let inner = block.inner(area);

            let scale = i32::from(SCALE.get());
            let w = i32::from(TETRIMINO_WIDTH);
            let h = i32::from(TETRIMINO_HEIGHT);

            let tetrimino_width = w * scale;
            let tetrimino_height = h * scale;

            let align_start = i32::from(tetrimino.hitbox().left) * tetrimino_width;

            for (dy, row) in tetrimino.cells().into_iter().enumerate() {
                for (dx, column) in row.into_iter().enumerate() {
                    if column == 0 {
                        continue;
                    }

                    let dx = i32::try_from(dx).unwrap();
                    let dy = i32::try_from(dy).unwrap();

                    let offset_x = dx * w * scale - align_start;
                    let x = i32::from(inner.x) + tetrimino_width + offset_x;

                    let offset_y = dy * h * scale;
                    let y = i32::from(inner.y) + tetrimino_height + offset_y;

                    self.render_game_tetrimino(buf, x, y, tetrimino.color());
                }
            }
        }

        block.render(area, buf);
    }

    fn render_game_left_controls(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Controls "))
            .padding(Padding::symmetric(2, 1))
            .border_set(border::DOUBLE);

        let controls = Paragraph::new(vec![
            Line::from("a,←: move left"),
            Line::from("d,→: move left"),
            Line::from("w,↑: hard drop"),
            Line::from("s,↓: soft drop"),
            Line::from("j,z: rotate left"),
            Line::from("k,x: rotate right"),
            Line::from("esc: quit"),
        ])
        .block(block);

        controls.render(area, buf);
    }

    fn render_game_playfield(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" TETRIS ").bold().centered())
            .border_set(border::DOUBLE);

        let inner = block.inner(area);

        self.render_game_playfield_cells(&inner, buf);
        self.render_game_playfield_active_tetrimino(&inner, buf);

        block.render(area, buf);
    }

    fn render_game_playfield_cells(&self, area: &Rect, buf: &mut Buffer) {
        for (dy, row) in self.game.playfield.cells.into_iter().enumerate() {
            for (dx, column) in row.into_iter().enumerate() {
                let color = match column {
                    Cell::Empty => {
                        continue;
                    }
                    Cell::Occupied(color) => color,
                };

                // X/Y offsets cannot be greater than the length/height of playfield (`u16`s).
                let dx = i32::try_from(dx).unwrap();
                let dy = i32::try_from(dy).unwrap();

                let scale = i32::from(SCALE.get());

                let x = i32::from(area.x) + (dx * i32::from(TETRIMINO_WIDTH) * scale);
                let y = i32::from(area.y) + (dy * i32::from(TETRIMINO_HEIGHT) * scale);

                self.render_game_tetrimino(buf, x, y, color);
            }
        }
    }

    fn render_game_playfield_active_tetrimino(&self, area: &Rect, buf: &mut Buffer) {
        for (dy, row) in self.game.active.tetrimino.cells().into_iter().enumerate() {
            for (dx, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                let scale = i32::from(SCALE.get());

                // X/Y offsets will never be greater than the length/height of a tetrimino (4x4).
                let dx = i32::try_from(dx).unwrap();
                let dy = i32::try_from(dy).unwrap();

                let offset_x = dx * i32::from(TETRIMINO_WIDTH) * scale;
                let width = self.game.active.x * i32::from(TETRIMINO_WIDTH) * scale;
                let x = i32::from(area.x) + offset_x + width;

                let offset_y = dy * i32::from(TETRIMINO_HEIGHT) * scale;
                let height = self.game.active.y * i32::from(TETRIMINO_HEIGHT) * scale;
                let y = i32::from(area.y) + offset_y + height;

                let color = self.game.active.tetrimino.color();

                self.render_game_tetrimino(buf, x, y, color);
            }
        }
    }

    fn render_game_game_over(&self, area: Rect, buf: &mut Buffer) {
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
                let style = if i == usize::from(self.game.selected as u8) {
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

    fn render_game_right(&self, area: Rect, buf: &mut Buffer) {
        let scale = u16::from(SCALE.get());

        let tetrimino_height = 4 * TETRIMINO_HEIGHT * scale;

        let [layout] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Start)
            .constraints(vec![Constraint::Length(25)])
            .areas(area);

        let [next, score] = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::SpaceBetween)
            .constraints(vec![
                Constraint::Length(tetrimino_height * 3 + 4),
                Constraint::Length(2 + 2 + 2),
            ])
            .areas(layout);

        self.render_game_right_next(next, buf);
        self.render_game_right_score(score, buf);
    }

    fn render_game_right_next(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Next "))
            .border_set(border::DOUBLE);

        let inner = block.inner(area);

        for (i, tetrimino) in self.game.bag.iter().enumerate().take(4) {
            let i = i32::try_from(i).unwrap();

            let scale = i32::from(SCALE.get());
            let w = i32::from(TETRIMINO_WIDTH);
            let h = i32::from(TETRIMINO_HEIGHT);

            let tetrimino_width = w * scale;
            let tetrimino_height = h * scale;
            let spacing = (tetrimino_height + 4) * i;

            let align_start = i32::from(tetrimino.hitbox().left) * tetrimino_width;

            for (dy, row) in tetrimino.cells().into_iter().enumerate() {
                for (dx, column) in row.into_iter().enumerate() {
                    if column == 0 {
                        continue;
                    }

                    let dx = i32::try_from(dx).unwrap();
                    let dy = i32::try_from(dy).unwrap();

                    let offset_x = dx * w * scale - align_start;
                    let x = i32::from(inner.x) + tetrimino_width + offset_x;

                    let offset_y = dy * h * scale;
                    let y = i32::from(inner.y) + tetrimino_height + offset_y + spacing;

                    self.render_game_tetrimino(buf, x, y, tetrimino.color());
                }
            }
        }

        block.render(area, buf);
    }

    fn render_game_right_score(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .padding(Padding::symmetric(2, 1))
            .border_set(border::DOUBLE);

        let score = Paragraph::new(vec![
            Line::from(format!("top: {}", self.game.top_score)),
            Line::from(format!("score: {}", self.game.score)),
        ])
        .block(block);

        score.render(area, buf);
    }

    /// Draw a Tetrimino on the screen.
    fn render_game_tetrimino(&self, buf: &mut Buffer, x: i32, y: i32, color: Color) {
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
