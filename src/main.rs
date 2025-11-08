mod board;
mod game;
mod render;
mod tetrimino;
mod tui;

use std::io;
use std::io::Write as _;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let result = game::Game::default().run(&mut terminal);

    if let Err(err) = tui::restore() {
        _ = writeln!(
            io::stderr(),
            "failed to restore terminal. Run `reset` or restart your terminal: {err}"
        );
    }

    result
}
