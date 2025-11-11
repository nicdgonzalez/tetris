use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Padding, Paragraph};
use ratatui::DefaultTerminal;

use crate::game::Game;
use crate::playfield::{PLAYFIELD_HEIGHT, PLAYFIELD_WIDTH};
use crate::render::{SCALE, TETRIMINO_HEIGHT, TETRIMINO_WIDTH};

/// Represents an event for the program to handle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// User input (via mouse, keyboard, etc.).
    Input(Event),

    /// Sent repeatedly at a fixed interval.
    Tick,
}

/// Represents the application's current task.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum State {
    /// User is playing the game.
    #[default]
    Running,

    /// User is ready to close the application.
    Quit,
}

#[derive(Debug, Clone, Default)]
pub struct App {
    state: State,
    game: Game,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the program.
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let (tx, rx) = mpsc::channel();

        // Forward input events through the channel.
        let tx_clone = tx.clone();
        thread::spawn(move || forward_input_events(tx_clone));

        // Start the application's internal clock.
        thread::spawn(move || start_ticker(tx));

        while !self.is_quitting() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&rx).wrap_err("failed to handle event")?;
        }

        self.game
            .save_score()
            .wrap_err("failed to save user's score")?;

        Ok(())
    }

    /// Renders the view to the screen.
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Handles tasks based on the user's input.
    fn handle_events(&mut self, rx: &Receiver<Message>) -> color_eyre::Result<()> {
        match rx.recv().wrap_err("failed to receive event")? {
            Message::Input(event) => match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            },
            Message::Tick => self.game.on_tick(),
        }

        Ok(())
    }

    /// Handle a keyboard press.
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.quit(),
            _ => self.game.on_key_event(key_event),
        }
    }

    /// Close the application.
    fn quit(&mut self) {
        self.state = State::Quit;
    }

    /// Whether the user has requested to close the application.
    fn is_quitting(&mut self) -> bool {
        self.game.exit || self.state == State::Quit
    }
}

/// Catches all input events and resends them through the channel.
fn forward_input_events(tx: Sender<Message>) {
    let timeout = Duration::from_millis(50); // Indicates how often to check for user input.

    loop {
        if let Ok(true) = event::poll(timeout) {
            let input_event = event::read().unwrap();

            tx.send(Message::Input(input_event))
                .expect("failed to send input event");
        }
    }
}

/// Sends events at a fixed interval through the channel.
fn start_ticker(tx: Sender<Message>) {
    let interval = Duration::from_millis(50); // Indicates how often game actions _can_ occur.

    loop {
        let next_tick = Instant::now() + interval;
        tx.send(Message::Tick).expect("failed to send tick event");

        let now = Instant::now();

        if now < next_tick {
            thread::sleep(next_tick - now);
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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

        self.render_left(left, buf);
        self.render_playfield(playfield, buf);
        self.render_right(right, buf);
    }
}

impl App {
    fn render_left(&self, area: Rect, buf: &mut Buffer) {
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

        self.render_left_hold(hold, buf);
        self.render_left_controls(controls, buf);
    }

    fn render_left_hold(&self, area: Rect, buf: &mut Buffer) {
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

                    self.render_tetrimino(buf, x, y, tetrimino.color());
                }
            }
        }

        block.render(area, buf);
    }

    fn render_left_controls(&self, area: Rect, buf: &mut Buffer) {
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

    fn render_playfield(&self, area: Rect, buf: &mut Buffer) {
        self.game.render(area, buf);
    }

    fn render_right(&self, area: Rect, buf: &mut Buffer) {
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

        self.render_right_next(next, buf);
        self.render_right_score(score, buf);
    }

    fn render_right_next(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from(" Next "))
            .border_set(border::DOUBLE);
        let inner = block.inner(area);

        for (i, tetrimino) in self.game.queue.iter().enumerate() {
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

                    self.render_tetrimino(buf, x, y, tetrimino.color());
                }
            }
        }

        block.render(area, buf);
    }

    fn render_right_score(&self, area: Rect, buf: &mut Buffer) {
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
