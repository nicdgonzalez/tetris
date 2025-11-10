use ratatui::prelude::*;

use crate::game::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Menu,
    Singleplayer,
}

pub struct App {
    pub view: View,
    game: Game,
}

impl Default for App {
    fn default() -> Self {
        Self {
            view: View::Menu,
            game: Game::default(),
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self.view {
            View::Menu => {
                // ...
            }
            View::Singleplayer => self.game.render(area, buf),
        }
    }
}
