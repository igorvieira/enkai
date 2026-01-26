use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, panic};

use crate::app::{AppState, ViewMode};
use crate::tui::{event::handle_events, views};

/// Run the TUI application
pub fn run_app(mut state: AppState) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set panic hook to ensure terminal cleanup even on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal (ignore errors since we're panicking anyway)
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        // Call the original panic hook
        original_hook(panic_info);
    }));

    // Run the main loop
    let result = run_loop(&mut terminal, &mut state);

    // Restore original panic hook
    let _ = panic::take_hook();

    // Restore terminal (this runs whether result is Ok or Err)
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut AppState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            let area = frame.size();

            match &state.view_mode {
                ViewMode::SplitPane { .. } => {
                    views::render_split_pane(frame, state, area);
                }
                ViewMode::RebaseActions => {
                    views::render_rebase_actions(frame, state, area);
                }
            }
        })?;

        handle_events(state)?;

        if state.should_quit {
            break;
        }
    }

    Ok(())
}
