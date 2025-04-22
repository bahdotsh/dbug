use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Panel types for the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelType {
    Source,
    Variables,
    CallStack,
    Watches,
    Breakpoints,
    Console,
}

/// Application state
pub struct App {
    /// Whether the application is running
    pub running: bool,
    /// Currently active panel
    pub active_panel: PanelType,
    /// Current scroll positions for each panel
    pub scroll_positions: HashMap<PanelType, usize>,
    /// Current debug session state
    pub debug_state: DebugState,
    /// Selected item in the current panel
    pub selected_item: Option<usize>,
}

/// Debug session state
pub enum DebugState {
    /// Not currently debugging
    Idle,
    /// Program is running
    Running,
    /// Program is paused at a breakpoint
    Paused {
        file: String,
        line: u32,
        function: String,
    },
}

impl App {
    /// Create a new application state
    pub fn new() -> Self {
        let mut scroll_positions = HashMap::new();
        for panel in &[
            PanelType::Source,
            PanelType::Variables,
            PanelType::CallStack,
            PanelType::Watches,
            PanelType::Breakpoints,
            PanelType::Console,
        ] {
            scroll_positions.insert(*panel, 0);
        }

        Self {
            running: true,
            active_panel: PanelType::Source,
            scroll_positions,
            debug_state: DebugState::Idle,
            selected_item: None,
        }
    }

    /// Handle a terminal event
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            _ => {}
        }
    }

    /// Handle a key event
    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            // Quit application with Ctrl+C or 'q'
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Char('q') => {
                self.running = false;
            }

            // Panel navigation with tab
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    PanelType::Source => PanelType::Variables,
                    PanelType::Variables => PanelType::CallStack,
                    PanelType::CallStack => PanelType::Watches,
                    PanelType::Watches => PanelType::Breakpoints,
                    PanelType::Breakpoints => PanelType::Console,
                    PanelType::Console => PanelType::Source,
                };
            }

            // Scroll up
            KeyCode::Up => {
                let scroll = self.scroll_positions.entry(self.active_panel).or_insert(0);
                if *scroll > 0 {
                    *scroll -= 1;
                }
            }

            // Scroll down
            KeyCode::Down => {
                let scroll = self.scroll_positions.entry(self.active_panel).or_insert(0);
                *scroll += 1;
            }

            // TODO: Add debugging control keys once we implement the flow control
            _ => {}
        }
    }

    /// Update application state
    pub fn update(&mut self) {
        // This method will be called on each loop iteration
        // It can be used to update application state or poll for debug events
    }

    /// Update application state with breakpoint information
    pub fn breakpoint_hit(&mut self, file: String, line: u32, function: String) {
        self.debug_state = DebugState::Paused {
            file,
            line,
            function,
        };
    }

    /// Continue execution until the next breakpoint
    fn continue_execution(&mut self) {
        // Create and send a continue response
        let response = crate::communication::DebuggerResponse::Continue;
        if let Err(e) = crate::communication::send_response(response) {
            eprintln!("Failed to send continue command: {}", e);
            return;
        }

        self.debug_state = DebugState::Running;
    }

    /// Step over the current line
    fn step_over(&mut self) {
        // Create and send a step over response
        let response = crate::communication::DebuggerResponse::StepOver;
        if let Err(e) = crate::communication::send_response(response) {
            eprintln!("Failed to send step over command: {}", e);
            return;
        }
    }

    /// Step into a function
    fn step_into(&mut self) {
        // Create and send a step into response
        let response = crate::communication::DebuggerResponse::StepInto;
        if let Err(e) = crate::communication::send_response(response) {
            eprintln!("Failed to send step into command: {}", e);
            return;
        }
    }

    /// Step out of the current function
    fn step_out(&mut self) {
        // Create and send a step out response
        let response = crate::communication::DebuggerResponse::StepOut;
        if let Err(e) = crate::communication::send_response(response) {
            eprintln!("Failed to send step out command: {}", e);
            return;
        }
    }
}
