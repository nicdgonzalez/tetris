use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use ratatui::DefaultTerminal;

use crate::board::{Board, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetrimino::{Orientation, Shape, Tetrimino, TETRIMINO_HEIGHT, TETRIMINO_WIDTH};

const SCALE: usize = 1;

// Top Left: Hold piece
// Top Right: Next piece
// Center: Game
// Bottom Right: Score/stats
// Bottom Left: Controls

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    board: Board,
    tetrimino: Tetrimino,
    x: u16,
    y: u16,
}

enum Message {
    UserInput(Event),
    Tick,
}

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
                Constraint::Length((BOARD_WIDTH * TETRIMINO_WIDTH * SCALE + 2) as u16),
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
                (BOARD_HEIGHT * TETRIMINO_HEIGHT * SCALE + 2) as u16,
            )])
            .areas(area);

        let title = Line::from(" Tetris ".bold());

        let game_block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        (&game_block).render(game, buf);

        let inner = game_block.inner(game);

        for (y, row) in self.board.cells.into_iter().enumerate() {
            for (x, column) in row.into_iter().enumerate() {
                let x = inner.x + (x as u16);
                let y = inner.y + (y as u16);

                for w in 1..=TETRIMINO_WIDTH {
                    if let Some(cell) = buf.cell_mut(Position::new(x + w as u16, y + w as u16)) {
                        if column.is_empty() {
                            continue;
                        }

                        cell.set_symbol("█")
                            .set_style(Style::default().fg(self.tetrimino.color()));
                    }
                }
            }
        }

        for (y, row) in self.tetrimino.cells().into_iter().enumerate() {
            for (x, column) in row.into_iter().enumerate() {
                let x = (inner.x) + self.x + (x as u16 * TETRIMINO_WIDTH as u16);
                let y = (inner.y) + self.y + (y as u16);

                for w in 1..=TETRIMINO_WIDTH {
                    if let Some(cell) = buf.cell_mut((x + w as u16, y)) {
                        if column == 0 {
                            continue;
                        }

                        cell.set_symbol("█")
                            .set_style(Style::default().fg(self.tetrimino.color()));
                    }
                }
            }
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        thread::spawn(move || loop {
            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                tx_clone
                    .send(Message::UserInput(event::read().unwrap()))
                    .expect("failed to send input event");
            }
        });

        // Start ticker
        let tick_interval = Duration::from_millis(1000); // The speed at which the tetrimino falls.
        thread::spawn(move || loop {
            let next_tick = Instant::now() + tick_interval;
            tx.send(Message::Tick).expect("failed to send tick event");

            let now = Instant::now();
            if now < next_tick {
                thread::sleep(next_tick - now);
            }
        });

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&rx).wrap_err("failed to handle event")?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self, rx: &mpsc::Receiver<Message>) -> color_eyre::Result<()> {
        match rx.recv().wrap_err("failed to receive event")? {
            Message::UserInput(input) => match input {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            },
            Message::Tick => self.handle_tick_event(),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('a') | KeyCode::Left => self.move_tetrimino_left(),
            KeyCode::Char('d') | KeyCode::Right => self.move_tetrimino_right(),
            KeyCode::Char('w') | KeyCode::Up => self.rotate_tetrimino_left(),
            KeyCode::Char('s') | KeyCode::Down => self.rotate_tetrimino_right(),
            KeyCode::Char('z') => self.switch_tetrimino(), // For testing
            _ => {}
        }
    }

    fn handle_tick_event(&mut self) {
        // Tetrimino falls as time goes by.
        self.y = self.y.saturating_add(1);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_tetrimino_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    fn move_tetrimino_right(&mut self) {
        self.x = self.x.saturating_add(1);
    }

    fn rotate_tetrimino_left(&mut self) {
        self.tetrimino.orientation = match self.tetrimino.orientation {
            Orientation::Up => Orientation::Left,
            Orientation::Left => Orientation::Down,
            Orientation::Down => Orientation::Right,
            Orientation::Right => Orientation::Up,
        };
    }

    fn rotate_tetrimino_right(&mut self) {
        self.tetrimino.orientation = match self.tetrimino.orientation {
            Orientation::Up => Orientation::Right,
            Orientation::Right => Orientation::Down,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up,
        };
    }

    fn switch_tetrimino(&mut self) {
        self.tetrimino.shape = match self.tetrimino.shape {
            Shape::I => Shape::O,
            Shape::O => Shape::T,
            Shape::T => Shape::S,
            Shape::S => Shape::Z,
            Shape::Z => Shape::J,
            Shape::J => Shape::L,
            Shape::L => Shape::I,
        };
        // Always return the tetrimino back to the upright position when switching.
        self.tetrimino.orientation = Orientation::Up;
    }
}
