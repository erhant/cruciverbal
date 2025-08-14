use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::time::{Duration, Instant};

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Quit,
}

pub struct EventHandler {
    last_tick: Instant,
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            last_tick: Instant::now(),
            tick_rate,
        }
    }

    pub fn next(&mut self) -> Result<AppEvent, Box<dyn std::error::Error>> {
        let timeout = self.tick_rate
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('q') {
                        return Ok(AppEvent::Quit);
                    }
                    Ok(AppEvent::Key(key_event))
                }
                _ => Ok(AppEvent::Tick),
            }
        } else {
            self.last_tick = Instant::now();
            Ok(AppEvent::Tick)
        }
    }
}