// CLI module for the dbug debugger

/// The current state of the debugger CLI
pub enum DebuggerState {
    /// The debugger is not active
    Inactive,
    /// The debugger is active and waiting for a command
    Active,
    /// The debugger is paused at a breakpoint
    AtBreakpoint,
    /// The debugger is running the target program
    Running,
}

/// Handles the CLI interface for the debugger
pub struct DebuggerCli {
    state: DebuggerState,
}

impl DebuggerCli {
    /// Create a new DebuggerCli instance
    pub fn new() -> Self {
        Self {
            state: DebuggerState::Inactive,
        }
    }
    
    /// Start the debugger CLI
    pub fn start(&mut self) {
        self.state = DebuggerState::Active;
        println!("Dbug debugger started. Type 'help' for a list of commands.");
    }
    
    /// Process a command from the user
    pub fn process_command(&mut self, command: &str) {
        match command.trim() {
            "help" => self.print_help(),
            "quit" | "q" => {
                println!("Exiting debugger");
                self.state = DebuggerState::Inactive;
            }
            _ => println!("Unknown command: {}. Type 'help' for a list of commands.", command),
        }
    }
    
    /// Print help information
    fn print_help(&self) {
        println!("Dbug Debugger Commands:");
        println!("  n, next            Step to the next line");
        println!("  s, step            Step into a function call");
        println!("  c, continue        Continue execution until the next breakpoint");
        println!("  p, print <expr>    Print the value of an expression");
        println!("  w, watch <expr>    Watch an expression for changes");
        println!("  q, quit            Quit the debugger");
        println!("  help               Show this help message");
    }
} 