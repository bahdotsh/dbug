use crate::tui::app::{App, DebugState, PanelType};
use std::vec::Vec;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Draw the UI
pub fn draw(f: &mut Frame, app: &App) {
    // Create the layout
    let chunks = create_layout(f.size());

    // Draw each panel
    draw_source_panel(f, app, chunks[0]);
    draw_variable_panel(f, app, chunks[1]);
    draw_call_stack_panel(f, app, chunks[2]);
    draw_watches_panel(f, app, chunks[3]);
    draw_breakpoints_panel(f, app, chunks[4]);
    draw_console_panel(f, app, chunks[5]);
}

/// Create the main layout
fn create_layout(area: Rect) -> Vec<Rect> {
    // Split the screen into two vertical sections
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70), // Top section for code and debugging
            Constraint::Percentage(30), // Bottom section for console
        ])
        .split(area);

    // Split the top section into columns
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Source code on the left
            Constraint::Percentage(50), // Debug info on the right
        ])
        .split(main_chunks[0]);

    // Split the right column into rows for different debug info
    let debug_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33), // Variables
            Constraint::Percentage(34), // Call stack
            Constraint::Percentage(33), // Watches
        ])
        .split(top_chunks[1]);

    // Split the bottom section into two for breakpoints and console
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Breakpoints
            Constraint::Percentage(70), // Console
        ])
        .split(main_chunks[1]);

    // Return all chunks in a flat array
    vec![
        top_chunks[0],    // Source code
        debug_chunks[0],  // Variables
        debug_chunks[1],  // Call stack
        debug_chunks[2],  // Watches
        bottom_chunks[0], // Breakpoints
        bottom_chunks[1], // Console
    ]
}

/// Draw the source code panel
fn draw_source_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Source Code";
    let block = create_block(title, app.active_panel == PanelType::Source);

    // Only show source code when the program is paused
    if let DebugState::Paused {
        file,
        line,
        function,
    } = &app.debug_state
    {
        // Get the source context
        let instrumenter = crate::instrumentation::Instrumenter::new(
            &std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy(),
        );
        let context_lines = 5; // Number of lines before and after current line

        // Try to get the source context from the instrumenter
        if let Ok(source_context) = instrumenter.get_source_context(file, *line, context_lines) {
            // Get the scroll offset
            let scroll = *app.scroll_positions.get(&PanelType::Source).unwrap_or(&0);

            // Create text with syntax highlighting for each line
            let mut all_lines = String::new();

            // Get sorted lines from start_line to end_line
            for line_num in source_context.start_line..=source_context.end_line {
                if let Some(source_line) = source_context.lines.get(&line_num) {
                    let is_current_line = line_num == *line;

                    if is_current_line {
                        all_lines.push_str(&format!("{:4} → {}\n", line_num, source_line));
                    } else {
                        all_lines.push_str(&format!("{:4}   {}\n", line_num, source_line));
                    }
                }
            }

            // Create a paragraph with the source code
            let paragraph = Paragraph::new(all_lines)
                .block(block)
                .scroll((scroll as u16, 0));

            f.render_widget(paragraph, area);
            return;
        }

        // Fallback if we can't get the actual source
        let text = format!(
            "File: {}\nLine: {}\nFunction: {}\n\nSource code not available",
            file, line, function
        );
        let paragraph = Paragraph::new(text).block(block);

        f.render_widget(paragraph, area);
        return;
    }

    // Default content when not paused
    let text = match &app.debug_state {
        DebugState::Running => "Program is running...".to_string(),
        DebugState::Idle => "No program is running".to_string(),
        _ => unreachable!(),
    };

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Draw the variables panel
fn draw_variable_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Variables";
    let block = create_block(title, app.active_panel == PanelType::Variables);

    // Placeholder content - would be replaced with actual variable data
    let items = vec![
        ListItem::new("var1 = 42"),
        ListItem::new("var2 = \"Hello, World!\""),
        ListItem::new("var3 = [1, 2, 3]"),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Draw the call stack panel
fn draw_call_stack_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Call Stack";
    let block = create_block(title, app.active_panel == PanelType::CallStack);

    // Placeholder content
    let items = vec![
        ListItem::new("main() at main.rs:10"),
        ListItem::new("foo() at foo.rs:25"),
        ListItem::new("bar() at bar.rs:42"),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Draw the watches panel
fn draw_watches_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Watches";
    let block = create_block(title, app.active_panel == PanelType::Watches);

    // Placeholder content
    let items = vec![
        ListItem::new("a + b = 15"),
        ListItem::new("x * y = 42"),
        ListItem::new("arr.len() = 3"),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Draw the breakpoints panel
fn draw_breakpoints_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Breakpoints";
    let block = create_block(title, app.active_panel == PanelType::Breakpoints);

    // Placeholder content
    let items = vec![
        ListItem::new("main.rs:15"),
        ListItem::new("foo.rs:42 (condition: x > 10)"),
        ListItem::new("bar.rs:23 (hit count: 2/3)"),
    ];

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

/// Draw the console panel
fn draw_console_panel(f: &mut Frame, app: &App, area: Rect) {
    let title = "Console";
    let block = create_block(title, app.active_panel == PanelType::Console);

    // Placeholder content
    let text = "Welcome to dbug!\nPress 'q' to quit, Tab to switch panels.\n> ";

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Create a block with the given title and highlight if active
fn create_block(title: &str, active: bool) -> Block {
    let style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(style)
}
