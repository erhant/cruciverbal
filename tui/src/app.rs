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

            // handle loading state - download puzzle
            if self.view == AppView::Game(GameView::Loading) {
                self.download_puzzle().await;
                continue;
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

    /// Download puzzle based on current selection state.
    async fn download_puzzle(&mut self) {
        use crate::game::GameView;
        use cruciverbal_providers::PuzzleProvider;

        let date = self.state.game.selection.date.clone();
        let provider = PuzzleProvider::ALL
            .get(self.state.game.selection.provider_idx)
            .copied()
            .unwrap_or_default();

        let result = match provider {
            PuzzleProvider::LovattsCryptic => {
                cruciverbal_providers::providers::lovatts_cryptic::download(&date).await
            }
        };

        match result {
            Ok(puzzle) => {
                self.state.game.puzzle = Some(puzzle);
                self.state.game.puzzle_date = Some(date);
                self.state.game.grid = None; // Will be built on first draw
                self.state.game.start_time = None; // Will be set on first draw
                self.view = AppView::Game(GameView::Playing);
            }
            Err(e) => {
                self.state.game.selection.error = Some(format!("Download failed: {}", e));
                self.view = AppView::Game(GameView::Selecting);
            }
        }
    }

    /// Renders the user interface.
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
