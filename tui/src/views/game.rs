#[derive(Default, Debug)]
pub struct GameState {}

use crate::{App, AppView};

impl App {
    pub fn draw_game(&mut self, frame: &mut ratatui::Frame) {
        let area = frame.area();
        frame.render_widget(
            ratatui::widgets::Paragraph::new("Game View - Press 'Q' to quit to menu").centered(),
            area,
        );
    }
}
