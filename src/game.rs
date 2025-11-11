use std::collections::VecDeque;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fmt, fs};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::playfield::{Cell, Playfield, PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};
use crate::tetrimino::{Cells, Orientation, RotateDirection, Shape, Tetrimino};

#[derive(Debug, Clone, Copy)]
pub struct ActiveTetrimino {
    pub tetrimino: Tetrimino,
    // Must be able to hold negative indices because some tetriminoes (e.g., `I` and `O`)
    // are offset from the edge of the grid.
    pub x: i32,
    pub y: i32,
}

impl Default for ActiveTetrimino {
    fn default() -> Self {
        let tetrimino = Tetrimino {
            shape: Shape::random(),
            orientation: Orientation::default(),
        };
        let playfield_center = PLAYFIELD_WIDTH / 2;
        let tetrimino_center = u16::try_from(tetrimino.cells().len()).unwrap() / 2;

        Self {
            tetrimino,
            x: (playfield_center - tetrimino_center).into(),
            y: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Playing,
    GameOver,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverOption {
    Restart,
    Exit,
}

impl fmt::Display for GameOverOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Restart => "Restart".fmt(f),
            Self::Exit => "Exit".fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub exit: bool,
    pub state: State,
    pub playfield: Playfield,
    pub active: ActiveTetrimino,
    pub next_tick: Option<Instant>,
    pub score: i32,
    pub hold: Option<Tetrimino>,
    pub can_hold: bool,
    pub selected: GameOverOption,
    pub queue: VecDeque<Tetrimino>,
    // TODO: Move to `App` after `Game` refactor.
    pub previous_scores: Vec<i32>,
    pub scores_txt: PathBuf,
    pub top_score: i32,
}

impl Default for Game {
    fn default() -> Self {
        // TODO: Move to `App` after `Game` refactor. These should not panic.
        let mut scores_txt = dirs::data_local_dir().expect("failed to get user's data directory");
        scores_txt.extend(["tetris", "scores.txt"]);
        let scores_txt = scores_txt;
        fs::create_dir_all(scores_txt.parent().unwrap())
            .expect("failed to create tetris data directory");

        let previous_scores = match fs::read_to_string(&scores_txt) {
            Ok(text) => text
                .lines()
                .filter_map(|line| line.parse::<i32>().ok())
                .collect(),
            Err(err) if err.kind() == ErrorKind::NotFound => Vec::new(),
            Err(err) => panic!("failed to get previous scores: {err}"),
        };

        let top_score = previous_scores.iter().cloned().max().unwrap_or_default();

        Self {
            exit: false,
            state: State::Playing,
            playfield: Playfield::default(),
            active: ActiveTetrimino::default(),
            next_tick: None,
            score: 0,
            selected: GameOverOption::Restart,
            hold: None,
            can_hold: true,
            queue: (0..4)
                .map(|_| Tetrimino {
                    shape: Shape::random(),
                    orientation: Orientation::default(),
                })
                .collect(),
            previous_scores,
            scores_txt,
            top_score,
        }
    }
}

impl Game {
    pub fn on_key_event(&mut self, key_event: KeyEvent) {
        if key_event.kind != KeyEventKind::Press {
            return;
        }

        self.handle_key_event(key_event);
    }

    pub fn on_tick(&mut self) {
        self.handle_tick_event();
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // TODO: Standard mappings for computer keyboards:
        // https://tetris.fandom.com/wiki/Tetris_Guideline
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Char('w') | KeyCode::Up | KeyCode::Char(' ') => match self.state {
                State::Playing => self.hard_drop(),
                State::GameOver => {
                    // In this case, there are only two options, but normally it would cycle
                    // through all of them. I don't think this is the best way to handle this,
                    // but... oh well.
                    self.selected = match self.selected {
                        GameOverOption::Restart => GameOverOption::Exit,
                        GameOverOption::Exit => GameOverOption::Restart,
                    };
                }
            },
            KeyCode::Char('a') | KeyCode::Left => {
                if self.state == State::Playing {
                    self.move_tetrimino_left();
                }
            }
            KeyCode::Char('s') | KeyCode::Down => match self.state {
                State::Playing => self.soft_drop(),
                // State::Playing => self.rotate_tetrimino(RotateDirection::Counterclockwise),
                State::GameOver => {
                    // In this case, there are only two options, but normally it would cycle
                    // through all of them. I don't think this is the best way to handle this,
                    // but... oh well.
                    self.selected = match self.selected {
                        GameOverOption::Restart => GameOverOption::Exit,
                        GameOverOption::Exit => GameOverOption::Restart,
                    };
                }
            },
            KeyCode::Char('d') | KeyCode::Right => {
                if self.state == State::Playing {
                    self.move_tetrimino_right();
                }
            }
            KeyCode::Char('z') | KeyCode::Char('j') => {
                self.rotate_tetrimino(RotateDirection::Counterclockwise)
            }
            KeyCode::Char('x') | KeyCode::Char('k') => {
                self.rotate_tetrimino(RotateDirection::Clockwise)
            }
            KeyCode::Char('c') | KeyCode::Char('q') => self.hold_tetrimino(),
            KeyCode::Enter => match self.state {
                State::Playing => {}
                State::GameOver => match self.selected {
                    GameOverOption::Restart => {
                        self.state = State::Playing;
                        self.playfield = Playfield::default();
                        self.next_tetrimino();
                        self.queue = (0..4)
                            .map(|_| Tetrimino {
                                shape: Shape::random(),
                                orientation: Orientation::default(),
                            })
                            .collect();
                    }
                    GameOverOption::Exit => {
                        self.exit();
                    }
                },
            },
            _ => {}
        }
    }

    fn handle_tick_event(&mut self) {
        if self.state != State::Playing {
            return;
        }

        let now = Instant::now();
        let interval = Duration::from_millis(500);

        match self.next_tick {
            Some(next_tick) => {
                if now < next_tick {
                    return;
                } else {
                    self.next_tick = Some(now + interval);
                }
            }
            None => {
                self.next_tick = Some(now + interval);
                return;
            }
        }

        let y = self.active.y.saturating_add(1);

        if self.tetrimino_fits(self.active.x, y, self.active.tetrimino.cells()) {
            self.active.y = y;
        } else {
            self.update_board();
            self.clear_finished_lines();
            self.next_tetrimino();
            self.can_hold = true;
        }
    }

    fn clear_finished_lines(&mut self) {
        let first_line = self.active.y + i32::from(self.active.tetrimino.hitbox().top);
        let last_line = self.active.y + i32::from(self.active.tetrimino.hitbox().bottom);

        for i in first_line..=last_line {
            // Scan the changed lines to see if any are now complete.
            let line = self.playfield.cells[usize::try_from(i).unwrap()];

            if line.iter().all(|c| !c.is_empty()) {
                self.score = self.score.saturating_add(10);
                let cells = self.playfield.cells;

                // Shift everything down by one line.
                for (y, row) in self
                    .playfield
                    .cells
                    .iter_mut()
                    .enumerate()
                    .rev()
                    .skip(usize::try_from(i32::from(PLAYFIELD_HEIGHT) - i - 1).unwrap())
                {
                    for (x, column) in row.iter_mut().enumerate() {
                        *column = cells[y.saturating_sub(1)][x];
                    }
                }
            }
        }
    }

    fn update_board(&mut self) {
        for (offset_y, row) in self.active.tetrimino.cells().into_iter().enumerate() {
            for (offset_x, column) in row.into_iter().enumerate() {
                if column == 0 {
                    continue;
                }

                // X/Y offsets cannot be greater than 4.
                assert_eq!(self.active.tetrimino.cells().len(), 4);
                let x = self.active.x + i32::try_from(offset_x).unwrap();
                assert_eq!(row.len(), 4);
                let y = self.active.y + i32::try_from(offset_y).unwrap();

                self.playfield.cells[usize::try_from(y).expect("invalid playboard row")]
                    [usize::try_from(x).expect("invalid playboard column")] =
                    Cell::Occupied(self.active.tetrimino.color());
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

                if (x < 0 || x >= PLAYFIELD_WIDTH.into()) || (y < 0 || y >= PLAYFIELD_HEIGHT.into())
                {
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
        let x = self.active.x.saturating_sub(1);

        if self.tetrimino_fits(x, self.active.y, self.active.tetrimino.cells()) {
            self.active.x = x;
        }
    }

    fn move_tetrimino_right(&mut self) {
        let x = self.active.x.saturating_add(1);

        if self.tetrimino_fits(x, self.active.y, self.active.tetrimino.cells()) {
            self.active.x = x;
        }
    }

    fn rotate_tetrimino(&mut self, direction: RotateDirection) {
        // TODO: I don't think I need to set and reset anymore; I can create a new
        // `ActiveTetrimino` and discard it if it wouldn't fit.

        let before = self.active;

        self.active.tetrimino = Tetrimino {
            shape: self.active.tetrimino.shape,
            orientation: self.active.tetrimino.orientation.rotate(direction),
        };

        let hitbox = self.active.tetrimino.hitbox();

        // Check if the rotated piece is in-bounds.
        if self.active.x - i32::from(hitbox.left) < 0 {
            self.active.x = 0 - i32::from(hitbox.left) + 1;
        }

        if self.active.x + i32::from(hitbox.right) >= PLAYFIELD_WIDTH.into() {
            self.active.x = i32::from(PLAYFIELD_WIDTH) - i32::from(hitbox.right) - 1;
        }

        if self.active.y + i32::from(hitbox.bottom) > PLAYFIELD_HEIGHT.into() {
            self.active.y = i32::from(PLAYFIELD_HEIGHT) - i32::from(hitbox.bottom) - 1;
        }

        // TODO: I'm pretty sure this is not supposed to be handled this way.
        if !self.tetrimino_fits(self.active.x, self.active.y, self.active.tetrimino.cells())
            && self.tetrimino_fits(
                self.active.x,
                self.active.y.saturating_sub(1),
                self.active.tetrimino.cells(),
            )
        {
            // Go up one if the block conflicts with another block.
            self.active.y = self.active.y.saturating_sub(1);
        }

        if !self.tetrimino_fits(self.active.x, self.active.y, self.active.tetrimino.cells()) {
            self.active = before;
        }
    }

    fn hold_tetrimino(&mut self) {
        if !self.can_hold {
            return;
        }

        match self.hold {
            Some(tetrimino) => {
                self.hold = Some(Tetrimino {
                    shape: self.active.tetrimino.shape,
                    orientation: Orientation::default(),
                });
                self.active = ActiveTetrimino {
                    tetrimino,
                    ..Default::default()
                };
            }
            None => {
                self.hold = Some(Tetrimino {
                    shape: self.active.tetrimino.shape,
                    orientation: Orientation::default(),
                });
                self.next_tetrimino();
            }
        }

        self.can_hold = false;
    }

    fn next_tetrimino(&mut self) {
        let next = ActiveTetrimino {
            tetrimino: self.queue.pop_front().unwrap(),
            ..Default::default()
        };

        if !self.tetrimino_fits(next.x, next.y, next.tetrimino.cells()) {
            self.state = State::GameOver;
        } else {
            self.active = next;
            self.queue.push_back(Tetrimino {
                shape: Shape::random(),
                orientation: Orientation::default(),
            });
        }
    }

    fn hard_drop(&mut self) {
        loop {
            let target_y = self.active.y.saturating_add(1);
            let bottom =
                i32::from(PLAYFIELD_HEIGHT) - i32::from(self.active.tetrimino.hitbox().bottom);

            if target_y > bottom {
                break;
            }

            if self.tetrimino_fits(self.active.x, target_y, self.active.tetrimino.cells()) {
                self.active.y = target_y;
                continue;
            }

            break;
        }

        self.next_tick = Some(Instant::now());
    }

    fn soft_drop(&mut self) {
        let y = self.active.y.saturating_add(1);

        if self.tetrimino_fits(self.active.x, y, self.active.tetrimino.cells()) {
            self.active.y = y;
        }
    }

    pub fn save_score(&self) -> color_eyre::Result<()> {
        if self.score < 1 {
            // TODO: Since score is an i32, if you manage to overflow the score, your score will
            // not be saved. :-)
            //
            // This will probably be fixed in the refactor, though I can't imagine anyone playing
            // long enough for this to ever happen.
            return Ok(());
        };

        assert!(self.scores_txt.parent().is_some_and(|dir| dir.exists()));
        let mut scores = self.previous_scores.clone();
        scores.push(self.score);
        fs::write(
            &self.scores_txt,
            scores
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        )?;
        Ok(())
    }
}
