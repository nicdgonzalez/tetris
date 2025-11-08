use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::DefaultTerminal;

use crate::board::{Board, Cell, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetrimino::{Orientation, Shape, Tetrimino};

#[derive(Debug, Default)]
pub struct Game {
    pub exit: bool,
    pub board: Board,
    pub tetrimino: Tetrimino,
    pub x: i16,
    pub y: i16,
    pub last_tick: Option<Instant>,
}

enum Message {
    UserInput(Event),
    Tick,
}

impl Game {
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
            KeyCode::Esc => self.exit(),
            KeyCode::Char('w') | KeyCode::Up => self.slam_tetrimino(),
            KeyCode::Char('a') | KeyCode::Left => self.move_tetrimino_left(),
            KeyCode::Char('s') | KeyCode::Down => self.fall_faster(),
            KeyCode::Char('d') | KeyCode::Right => self.move_tetrimino_right(),
            KeyCode::Char('j') => self.rotate_tetrimino_left(),
            KeyCode::Char('k') => self.rotate_tetrimino_right(),
            KeyCode::Char('q') => self.switch_tetrimino(), // For testing
            _ => {}
        }
    }

    fn handle_tick_event(&mut self) {
        let now = Instant::now();

        if self.last_tick.is_none() {
            self.last_tick = Some(now);
            return;
        }

        // Tetrimino falls one block every second.
        if now < self.last_tick.unwrap() + Duration::from_millis(1000) {
            return;
        }

        let y = self.y.saturating_add(1);

        // Replace BOARD_HEIGHT with occuppied board cells.
        //
        // 1 0 0 0 0 0
        // 1 1 0 0 0 0
        // 1 2 2 2 2 0
        //
        // I can use the bottom_right hitbox and scan the whole bottom row for collisions... maybe?
        //
        // 1 0 0 0
        // 1 1 0 0
        // 1 2 0 0
        // 0 2 0 0
        // 0 2 0 0
        // 0 2 0 0
        //
        // I don't think there is a way to get around scanning all of the cells and checking
        // collisions.
        if self.tetrimino_fits(self.x, y) {
            //if (y + self.tetrimino.hitbox_bottom_right().column) < BOARD_HEIGHT as i16 {
            self.y = y;
        } else {
            // Save to board.
            self.save_board();
            self.switch_tetrimino();
        }

        // TODO: Check if any lines were cleared.

        self.last_tick = Some(now);
    }

    fn save_board(&mut self) {
        for (offset_y, row) in self.tetrimino.cells().into_iter().enumerate() {
            for (offset_x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                let x = self.x + offset_x as i16;
                let y = self.y + offset_y as i16;

                self.board.cells[y as usize][x as usize] = Cell::Occupied(self.tetrimino.color());
            }
        }
    }

    // TODO: Better error handling... the number of times this program has panicked... :-)
    fn tetrimino_fits(&self, target_x: i16, target_y: i16) -> bool {
        for (offset_y, row) in self.tetrimino.cells().into_iter().enumerate() {
            for (offset_x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue; // There is no block to collide with.
                }

                let x = target_x + offset_x as i16;
                let y = target_y + offset_y as i16;

                if (x < 0 || x >= BOARD_WIDTH as i16) || (y < 0 || y >= BOARD_HEIGHT as i16) {
                    return false;
                } else if let Cell::Occupied(_) = self.board.cells[y as usize][x as usize] {
                    return false;
                }
            }
        }

        true
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_tetrimino_left(&mut self) {
        let x = self.x.saturating_sub(1);

        if self.tetrimino_fits(x, self.y) {
            self.x = x;
        }
    }

    fn move_tetrimino_right(&mut self) {
        let x = self.x.saturating_add(1);

        if self.tetrimino_fits(x, self.y) {
            self.x = x;
        }
    }

    fn rotate_tetrimino_left(&mut self) {
        self.tetrimino.orientation = match self.tetrimino.orientation {
            Orientation::Up => Orientation::Right,
            Orientation::Right => Orientation::Down,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up,
        };

        let after = (
            self.tetrimino.hitbox_top_left(),
            self.tetrimino.hitbox_bottom_right(),
        );

        // Check if the rotated piece is in-bounds.
        if self.x - after.0.column < 0 {
            self.x = after.0.column;
        }

        if self.x + after.1.column >= BOARD_WIDTH as i16 {
            self.x = BOARD_WIDTH as i16 - after.1.column - 1;
        }

        if self.y + after.1.row > BOARD_HEIGHT as i16 {
            self.y = BOARD_HEIGHT as i16 - after.1.row - 1;
        }
    }

    fn rotate_tetrimino_right(&mut self) {
        let orientation = match self.tetrimino.orientation {
            Orientation::Up => Orientation::Left,
            Orientation::Left => Orientation::Down,
            Orientation::Down => Orientation::Right,
            Orientation::Right => Orientation::Up,
        };

        // TODO: Test if turning would conflict with other tetriminos.
        // TODO: Create a `rotate_clockwise` and `rotate_counterclockwise` method on Tetrimino
        // TODO: Decouple the tetrimino_fits function from the Game type.

        self.tetrimino.orientation = orientation;

        let after = (
            self.tetrimino.hitbox_top_left(),
            self.tetrimino.hitbox_bottom_right(),
        );

        // TODO: Check that the rotation doesn't enter another block.

        // Check if the rotated piece is in-bounds.
        if self.x - after.0.column < 0 {
            self.x = after.0.column;
        }

        if self.x + after.1.column >= BOARD_WIDTH as i16 {
            self.x = BOARD_WIDTH as i16 - after.1.column - 1;
        }

        if self.y + after.1.row > BOARD_HEIGHT as i16 {
            self.y = BOARD_HEIGHT as i16 - after.1.row - 1;
        }

        if !self.tetrimino_fits(self.x, self.y)
            && self.tetrimino_fits(self.x, self.y.saturating_sub(1))
        {
            // Go up one if the block conflicts with another block.
            self.y = self.y.saturating_sub(1);
        }
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
        self.x = (BOARD_WIDTH as i16).checked_div(2).unwrap();
    }

    fn slam_tetrimino(&mut self) {
        loop {
            let target_y = self.y.saturating_add(1);

            if target_y > BOARD_HEIGHT as i16 - self.tetrimino.hitbox_bottom_right().column {
                break;
            }

            if self.tetrimino_fits(self.x, target_y) {
                self.y = target_y;
                continue;
            }

            break;
        }
    }

    fn fall_faster(&mut self) {
        let y = self.y.saturating_add(1);

        if self.tetrimino_fits(self.x, y) {
            self.y = y;
        }
    }
}
