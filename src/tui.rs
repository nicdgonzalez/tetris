use std::{io, panic};

use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

pub fn init() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();
    Terminal::new(CrosstermBackend::new(io::stdout()))
}

fn set_panic_hook() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        hook(panic_info);
    }));
}

pub fn restore() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
