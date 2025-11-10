use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use color_eyre::eyre::Context as _;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::DefaultTerminal;

use crate::game::Game;

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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.game.render(area, buf)
    }
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
