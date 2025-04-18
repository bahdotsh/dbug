// Runtime engine module for the dbug debugger

/// A breakpoint in the code
pub struct Breakpoint {
    /// The file containing the breakpoint
    pub file: String,
    /// The line number of the breakpoint
    pub line: u32,
    /// The column number of the breakpoint
    pub column: u32,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// The condition for the breakpoint to trigger (if any)
    pub condition: Option<String>,
}

impl Breakpoint {
    /// Create a new breakpoint
    pub fn new(file: &str, line: u32, column: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            enabled: true,
            condition: None,
        }
    }
    
    /// Set a condition for the breakpoint
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
}

/// The runtime engine for the debugger
pub struct DebuggerRuntime {
    /// Whether the debugger is running
    pub running: bool,
    /// The breakpoints that have been set
    pub breakpoints: Vec<Breakpoint>,
}

impl DebuggerRuntime {
    /// Create a new DebuggerRuntime
    pub fn new() -> Self {
        Self {
            running: false,
            breakpoints: Vec::new(),
        }
    }
    
    /// Start the debugger runtime
    pub fn start(&mut self) {
        self.running = true;
        println!("Debugger runtime started");
    }
    
    /// Stop the debugger runtime
    pub fn stop(&mut self) {
        self.running = false;
        println!("Debugger runtime stopped");
    }
    
    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
        println!("Added breakpoint at {}:{}:{}", breakpoint.file, breakpoint.line, breakpoint.column);
        self.breakpoints.push(breakpoint);
    }
} 