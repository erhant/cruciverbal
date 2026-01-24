use crate::{
    game::{GameState, GameView},
    menu::MenuState,
};
use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use std::time::Duration;

#[derive(Default, Clone, Debug, PartialEq)]
pub enum AppView {
    #[default]
    Menu,
    Game(GameView),
}

#[derive(Default, Debug)]
pub struct AppState {
    pub menu: MenuState,
    pub game: GameState,
}

/// 35 FPS = 1000ms / 35
const FPS_RATE: Duration = Duration::from_millis(1000 / 35);

pub struct App {
    /// Active application view.
    pub view: AppView,
    /// Application state.
    ///
    /// This is shared among all views.
    pub state: AppState,
    /// Is the application running?
    pub is_running: bool,
    /// Event stream.
    pub event_stream: EventStream,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            is_running: false,
            event_stream: EventStream::new(),
            view: AppView::Menu,
            state: AppState::default(),
        }
    }

    /// Set the active view.
    pub fn set_view(&mut self, view: AppView) {
        self.view = view;
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: ratatui::DefaultTerminal) -> Result<()> {
        self.is_running = true;

        // create a ticker for animation updates
        let mut interval = tokio::time::interval(FPS_RATE);

        while self.is_running {
            // draw first (to disguise async stuff in ticks)
            terminal.draw(|frame| self.draw(frame))?;

            // process ticks
            if self.state.game.puzzle.is_none() {
                // FIXME: mock download game
                let date = "2025-12-08";
                self.state.game.puzzle =
                    cruciverbal_providers::providers::lovatts_cryptic::download(date)
                        .await
                        .ok();
                if self.state.game.puzzle.is_some() {
                    self.state.game.puzzle_date = Some(date.to_string());
                }
            }

            // handle events with timeout to allow animation updates
            tokio::select! {
                _ = interval.tick() => {
                    // trigger a redraw for animation by looping
                    continue;
                }
                result = self.handle_crossterm_events() => {
                    result?;
                }
            }
        }

        Ok(())
    }

    /// Renders the user interface.
    ///
    /// TODO: separate footer and header here, and give the frame only the body area.
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        match self.view.clone() {
            AppView::Menu => self.draw_menu(frame),
            AppView::Game(view) => self.draw_game(view, frame),
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    async fn handle_crossterm_events(&mut self) -> Result<()> {
        use crossterm::event::{Event, KeyEventKind, KeyModifiers};
        use futures::{FutureExt, StreamExt};

        let event = self.event_stream.next().fuse().await;
        match event {
            Some(Ok(evt)) => match evt {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    use crossterm::event::KeyCode;

                    // application-wide CTRL+C handler
                    if matches!(
                        (key.modifiers, key.code),
                        (
                            KeyModifiers::CONTROL,
                            KeyCode::Char('c') | KeyCode::Char('C')
                        )
                    ) {
                        self.quit();
                        return Ok(());
                    };

                    match self.view.clone() {
                        AppView::Menu => self.handle_menu_input(key),
                        AppView::Game(view) => self.handle_game_input(view, key),
                    }
                }
                Event::Mouse(_) => {} // no mouse events
                Event::Resize(_, _) => {}
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.is_running = false;
    }
}
