use crate::prelude::*;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Stdout};
use tui::{backend::CrosstermBackend, Terminal};

/// Terminal type alias
pub type DbugTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Setup the terminal for TUI rendering
pub fn setup_terminal() -> DbugResult<DbugTerminal> {
    // Set up terminal
    enable_raw_mode()
        .map_err(|e| DbugError::TuiError(format!("Failed to enable raw mode: {}", e)))?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| DbugError::TuiError(format!("Failed to enter alternate screen: {}", e)))?;

    // Create terminal with crossterm backend
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)
        .map_err(|e| DbugError::TuiError(format!("Failed to create terminal: {}", e)))?;

    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore_terminal() -> DbugResult<()> {
    // Restore terminal
    disable_raw_mode()
        .map_err(|e| DbugError::TuiError(format!("Failed to disable raw mode: {}", e)))?;

    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| DbugError::TuiError(format!("Failed to leave alternate screen: {}", e)))?;

    Ok(())
}
