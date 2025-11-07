use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::DefaultTerminal;

use crate::board::{Board, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetrimino::{Orientation, Shape, Tetrimino};

// Top Left: Hold piece
// Top Right: Next piece
// Center: Game
// Bottom Right: Score/stats
// Bottom Left: Controls

#[derive(Debug, Default)]
pub struct App {
    pub exit: bool,
    pub board: Board,
    pub tetrimino: Tetrimino,
    pub x: u16,
    pub y: u16,
    pub last_tick: Option<Instant>,
}

enum Message {
    UserInput(Event),
    Tick,
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
        let tick_interval = Duration::from_millis(100); // The speed at which the tetrimino falls.
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
            KeyCode::Char(' ') => self.slam_tetrimino(),
            _ => {}
        }
    }

    fn handle_tick_event(&mut self) {
        let now = Instant::now();

        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }

        if now < self.last_tick.unwrap() + Duration::from_millis(1000) {
            return;
        }

        // Tetrimino falls one block at every tick.
        //
        // TODO: We will probably need to render ticks more often, and track
        // the last_tick/current_tick for the falling piece. that way we can render a line complete
        // or something with fancy graphics later without it rendering at 1 animation per second.
        let y = self.y.saturating_add(1);

        // TODO: Need to account for size of tetrimino (and scale).
        if y < BOARD_HEIGHT {
            self.y = y;
        }

        self.last_tick = Some(now);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_tetrimino_left(&mut self) {
        self.x = self.x.saturating_sub(1);
    }

    fn move_tetrimino_right(&mut self) {
        let x = self.x.saturating_add(1);

        // TODO: Handle collision with wall.
        if x < BOARD_WIDTH {
            self.x = x;
        }
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
        self.y = 0;
    }

    fn slam_tetrimino(&mut self) {
        self.y = BOARD_HEIGHT - 1;
        // TODO: Save to board after slam, then switch to next tetrimino.
    }
}
