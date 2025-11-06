use std::io;

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use ratatui::DefaultTerminal;

use crate::board::{Board, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetrimino::{Tetrimino, TETRIMINO_HEIGHT, TETRIMINO_WIDTH};

// Top Left: Hold piece
// Top Right: Next piece
// Center: Game
// Bottom Right: Score/stats
// Bottom Left: Controls

#[derive(Debug, Default)]
pub struct App {
    counter: u8,
    exit: bool,
    board: Board,
    current_tetrimino: Tetrimino,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [hold, game, next] = Layout::horizontal([
            Constraint::Fill(0),
            Constraint::Length(
                u16::try_from(BOARD_WIDTH * TETRIMINO_WIDTH).expect("usize overflowed u16"),
            ),
            Constraint::Fill(0),
        ])
        .vertical_margin((BOARD_HEIGHT * TETRIMINO_HEIGHT) as u16)
        .areas(area);

        self.render_game(game, buf);
    }
}

impl App {
    fn render_game(&self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Tetris ".bold());

        let game_block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        (&game_block).render(area, buf);

        let inner = game_block.inner(area);

        // At this time, I have successfully rendered the T tetrimino in the square (though it all
        // looks a bit small... :-)
        //
        // Storing the Shape and Matrix on the `shape` is long and obnoxious to type out... I need
        // to fix that.
        //
        // I want to be able to render two horizontal cells for each square... though I can't
        // figure out how to do that yet. If I just add +1 to the X-axis, if there is already a
        // block there, it doesn't add an extra one. Maybe I need to draw directly on the board and
        // render that at 2x scale?
        for (y, row) in self
            .current_tetrimino
            .shape
            .get(self.current_tetrimino.orientation)
            .into_iter()
            .enumerate()
        {
            for (x, column) in row.into_iter().enumerate() {
                let x = inner.x + x as u16;
                let y = inner.y + y as u16;

                if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                    if column == 0 {
                        continue;
                    }

                    cell.set_symbol("█")
                        .set_style(Style::default().fg(self.current_tetrimino.shape.color()));
                }
            }
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().wrap_err("failed to handle event")?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn decrement_counter(&mut self) {
        self.counter = self.counter.saturating_sub(1);
    }

    fn increment_counter(&mut self) {
        self.counter = self.counter.saturating_add(1);
    }
}

// Ignore -- it's from the Ratatui tutorial :-)
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn render() {
        let app = App::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 4));

        app.render(buf.area, &mut buf);

        let mut expected = Buffer::with_lines(vec![
            "┏━━━━━━━━━━━━━━━━━━━━━Tetris━━━━━━━━━━━━━━━━━━━━━┓",
            "┃                    Value: 0                    ┃",
            "┃                                                ┃",
            "┗━━━━━━━━━━━━━━━━━━━ quit <q> ━━━━━━━━━━━━━━━━━━━┛",
        ]);
        let title_style = Style::new().bold();
        let counter_style = Style::new().yellow();
        let key_style = Style::new().blue().bold();
        expected.set_style(Rect::new(14, 0, 22, 1), title_style);
        expected.set_style(Rect::new(28, 1, 1, 1), counter_style);
        expected.set_style(Rect::new(13, 3, 6, 1), key_style);
        expected.set_style(Rect::new(30, 3, 7, 1), key_style);
        expected.set_style(Rect::new(43, 3, 4, 1), key_style);

        assert_eq!(buf, expected);
    }
    #[test]
    fn handle_key_event() -> io::Result<()> {
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into());
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into());
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into());
        assert!(app.exit);

        Ok(())
    }
}
