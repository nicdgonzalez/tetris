use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::DefaultTerminal;

use crate::board::{Cell, Playfield, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetrimino::{Cells, Orientation, RotateDirection, Shape, Tetrimino};

#[derive(Debug)]
pub struct Game {
    pub exit: bool,
    pub playfield: Playfield,
    pub tetrimino: Tetrimino,
    // Must be able to hold negative indices because some tetriminoes (e.g., `I` and `O`)
    // are offset from the edge of the grid.
    pub x: i32,
    pub y: i32,
    pub last_tick: Option<Instant>,
}

impl Default for Game {
    fn default() -> Self {
        let tetrimino = Tetrimino {
            shape: Shape::T,
            orientation: Orientation::Up,
        };

        Self {
            exit: false,
            playfield: Playfield::default(),
            tetrimino,
            x: i32::from(BOARD_WIDTH).checked_div(2).unwrap()
                - i32::try_from(tetrimino.cells().len().checked_div(2).unwrap()).unwrap(),
            y: 0,
            last_tick: None,
        }
    }
}

enum Message {
    Input(Event),
    Tick,
}

impl Game {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (tx, rx) = mpsc::channel();

        let tx_clone = tx.clone();
        thread::spawn(move || loop {
            if let Ok(true) = event::poll(Duration::from_millis(50)) {
                tx_clone
                    .send(Message::Input(event::read().unwrap()))
                    .expect("failed to send input event");
            }
        });

        // Start ticker
        let tick_interval = Duration::from_millis(100);
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
            Message::Input(input) => match input {
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
        // Standard mappings for computer keyboards:
        // https://tetris.fandom.com/wiki/Tetris_Guideline
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Char('w') | KeyCode::Up => self.slam_tetrimino(),
            KeyCode::Char('a') | KeyCode::Left => self.move_tetrimino_left(),
            KeyCode::Char('s') | KeyCode::Down => self.fall_faster(),
            KeyCode::Char('d') | KeyCode::Right => self.move_tetrimino_right(),
            KeyCode::Char('j') => self.rotate_tetrimino(RotateDirection::Counterclockwise),
            KeyCode::Char('k') => self.rotate_tetrimino(RotateDirection::Clockwise),
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

        if now < self.last_tick.unwrap() + Duration::from_millis(500) {
            return;
        }

        self.last_tick = Some(now);

        let y = self.y.saturating_add(1);

        if self.tetrimino_fits(self.x, y, self.tetrimino.cells()) {
            self.y = y;
        } else {
            // Save to board.
            self.update_board();
            self.switch_tetrimino();
        }

        // TODO: Check if any lines were cleared.
    }

    fn update_board(&mut self) {
        for (offset_y, row) in self.tetrimino.cells().into_iter().enumerate() {
            for (offset_x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                // X/Y offsets cannot be greater than 4.
                assert_eq!(self.tetrimino.cells().len(), 4);
                let x = self.x + i32::try_from(offset_x).unwrap();
                assert_eq!(row.len(), 4);
                let y = self.y + i32::try_from(offset_y).unwrap();

                self.playfield.cells[usize::try_from(y).expect("invalid playboard row")]
                    [usize::try_from(x).expect("invalid playboard column")] =
                    Cell::Occupied(self.tetrimino.color());
            }
        }
    }

    fn tetrimino_fits(&self, target_x: i32, target_y: i32, cells: Cells) -> bool {
        for (offset_y, row) in cells.into_iter().enumerate() {
            for (offset_x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue; // There is no block to collide with.
                }

                // X/Y offsets cannot be greater than 4.
                assert_eq!(cells.len(), 4);
                let x = target_x + i32::try_from(offset_x).unwrap();
                assert_eq!(row.len(), 4);
                let y = target_y + i32::try_from(offset_y).unwrap();

                if (x < 0 || x >= BOARD_WIDTH.into()) || (y < 0 || y >= BOARD_HEIGHT.into()) {
                    return false;
                } else if let Cell::Occupied(_) = self.playfield.cells
                    [usize::try_from(y).expect("invalid playfield row")]
                    [usize::try_from(x).expect("invalid playfield column")]
                {
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

        if self.tetrimino_fits(x, self.y, self.tetrimino.cells()) {
            self.x = x;
        }
    }

    fn move_tetrimino_right(&mut self) {
        let x = self.x.saturating_add(1);

        if self.tetrimino_fits(x, self.y, self.tetrimino.cells()) {
            self.x = x;
        }
    }

    fn rotate_tetrimino(&mut self, direction: RotateDirection) {
        let before = self.tetrimino;

        self.tetrimino = Tetrimino {
            shape: self.tetrimino.shape,
            orientation: self.tetrimino.orientation.rotate(direction),
        };

        let hitbox = self.tetrimino.hitbox();

        // Check if the rotated piece is in-bounds.
        if self.x - i32::from(hitbox.left) < 0 {
            self.x = 0 - i32::from(hitbox.left) + 1;
        }

        if self.x + i32::from(hitbox.right) >= BOARD_WIDTH.into() {
            self.x = i32::from(BOARD_WIDTH) - i32::from(hitbox.right) - 1;
        }

        if self.y + i32::from(hitbox.bottom) > BOARD_HEIGHT.into() {
            self.y = i32::from(BOARD_HEIGHT) - i32::from(hitbox.bottom) - 1;
        }

        if !self.tetrimino_fits(self.x, self.y, self.tetrimino.cells())
            && self.tetrimino_fits(self.x, self.y.saturating_sub(1), self.tetrimino.cells())
        {
            // Go up one if the block conflicts with another block.
            self.y = self.y.saturating_sub(1);
        }

        if !self.tetrimino_fits(self.x, self.y, self.tetrimino.cells()) {
            self.tetrimino = before;
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
        self.x = i32::from(BOARD_WIDTH).checked_div(2).unwrap()
            - i32::try_from(self.tetrimino.cells().len().checked_div(2).unwrap()).unwrap();
    }

    fn slam_tetrimino(&mut self) {
        loop {
            let target_y = self.y.saturating_add(1);
            let bottom = i32::from(BOARD_HEIGHT) - i32::from(self.tetrimino.hitbox().bottom);

            if target_y > bottom {
                break;
            }

            if self.tetrimino_fits(self.x, target_y, self.tetrimino.cells()) {
                self.y = target_y;
                continue;
            }

            break;
        }
    }

    fn fall_faster(&mut self) {
        let y = self.y.saturating_add(1);

        if self.tetrimino_fits(self.x, y, self.tetrimino.cells()) {
            self.y = y;
        }
    }
}
