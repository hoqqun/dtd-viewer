mod state;
mod view;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;

use crate::model::Dtd;
use state::AppState;

pub fn run(dtd: Dtd) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut state = AppState::new(dtd);

    loop {
        terminal.draw(|f| view::render(f, &state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if state.search_mode {
                match key.code {
                    KeyCode::Esc => state.cancel_search(),
                    KeyCode::Enter => state.finish_search(),
                    KeyCode::Backspace => state.search_backspace(),
                    KeyCode::Char(c) => state.search_input(c),
                    _ => {}
                }
                continue;
            }

            match state.overlay {
                Some(_) => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('e') | KeyCode::Char('a') => {
                        state.close_overlay();
                    }
                    _ => {}
                },
                None => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                    KeyCode::Enter | KeyCode::Right => state.expand(),
                    KeyCode::Left => state.collapse(),
                    KeyCode::Char('/') => state.start_search(),
                    KeyCode::Char('e') => state.show_entities(),
                    KeyCode::Char('a') => state.show_attributes(),
                    KeyCode::Char('n') => state.next_search_match(),
                    KeyCode::Char('N') => state.prev_search_match(),
                    _ => {}
                },
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
