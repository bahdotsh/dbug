pub mod app;
pub mod event;
pub mod terminal;
pub mod ui;

use crate::prelude::*;
use std::time::Duration;

/// Initialize and run the TUI application
pub fn run() -> DbugResult<()> {
    // Initialize terminal
    let mut terminal = terminal::setup_terminal()?;

    // Create app state
    let mut app = app::App::new();

    // Create event handler
    let mut events = event::Events::new(Duration::from_millis(100));

    // Main loop
    while app.running {
        // Draw the UI
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle events
        if let Some(event) = events.next()? {
            app.handle_event(event);
        }

        // Check for debugger messages
        match crate::communication::check_for_messages() {
            Ok(Some(msg)) => {
                match msg {
                    crate::communication::DebuggerMessage::BreakpointHit {
                        file,
                        line,
                        column: _,
                        function,
                    } => {
                        app.breakpoint_hit(file, line, function);
                    }
                    crate::communication::DebuggerMessage::FunctionEntered {
                        function,
                        file,
                        line,
                    } => {
                        // Update the current function
                        app.debug_state = app::DebugState::Paused {
                            file,
                            line,
                            function,
                        };
                    }
                    crate::communication::DebuggerMessage::FunctionExited { function: _ } => {
                        // Just note the function exit
                    }
                    crate::communication::DebuggerMessage::VariableChanged {
                        name: _,
                        type_name: _,
                        value: _,
                        is_mutable: _,
                    } => {
                        // Update variables
                        // (We'll implement this later in a more complete version)
                    }
                    _ => {
                        // Other message types
                    }
                }
            }
            Err(e) => {
                eprintln!("Error checking for messages: {}", e);
            }
            _ => {}
        }

        // Update state
        app.update();
    }

    // Restore terminal
    terminal::restore_terminal()?;

    Ok(())
}
