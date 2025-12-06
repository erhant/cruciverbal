use crate::{game::GameState, menu::MenuState};
use color_eyre::eyre::Result;
use crossterm::event::EventStream;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum AppView {
    Menu,
    Game,
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
    pub fn new() -> Result<Self> {
        Self::new_at_view(AppView::Menu)
    }

    pub fn new_at_view(view: AppView) -> Result<Self> {
        Ok(Self {
            is_running: false,
            event_stream: EventStream::new(),
            view,
            state: AppState::default(),
        })
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
            // TODO: no ticks yet

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
            AppView::Game => self.draw_game(frame),
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

                    match &self.view.clone() {
                        AppView::Menu => self.handle_menu_input(key),
                        AppView::Game => todo!(),
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
