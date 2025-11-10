mod app;
mod game;
mod playfield;
mod render;
mod tetrimino;
mod tui;

use std::io;
use std::io::Write as _;

use app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = tui::init()?;
    let result = App::default().run(&mut terminal);

    if let Err(err) = tui::restore() {
        _ = writeln!(
            io::stderr(),
            "failed to restore terminal. Run `reset` or restart your terminal: {err}"
        );
    }

    result
}
