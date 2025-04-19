// CLI module for the dbug debugger

use std::collections::HashMap;
use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use termcolor::Color;
use std::io::{self, Read, Write};
use std::fs;
use std::path::Path;
use std::process::exit;

// Define local versions of the types we need
// In a real implementation, these would be properly imported from the runtime module
#[derive(Debug, Clone)]
pub enum FlowControl {
    Continue,
    StepOver,
    StepInto,
    StepOut,
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Struct(HashMap<String, String>),
    Array(Vec<String>),
    Null,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub type_name: String,
    pub value: VariableValue,
    pub scope_level: u32,
    pub is_mutable: bool,
}

#[derive(Debug)]
pub struct DebuggerRuntime {
    // For simplicity, we'll use basic structures here
    breakpoints: Vec<(String, u32, bool)>, // (file, line, enabled)
    watches: Vec<(String, u32)>,           // (expression, id)
    variables: HashMap<String, Variable>,
    current_frame: usize,
    frames: Vec<(String, String, u32)>,    // (function, file, line)
}

impl DebuggerRuntime {
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            watches: Vec::new(),
            variables: HashMap::new(),
            current_frame: 0,
            frames: Vec::new(),
        }
    }
    
    pub fn start(&mut self) {
        // Initialize runtime
    }
    
    pub fn stop(&mut self) {
        // Clean up runtime
    }
    
    pub fn add_breakpoint(&mut self, file: &str, line: u32, _column: u32) {
        println!("Setting breakpoint at {}:{}", file, line);
        self.breakpoints.push((file.to_string(), line, true));
    }
    
    pub fn remove_breakpoint(&mut self, id: usize) -> bool {
        if id < self.breakpoints.len() {
            self.breakpoints.remove(id);
            true
        } else {
            false
        }
    }
    
    pub fn toggle_breakpoint(&mut self, id: usize, enabled: bool) -> bool {
        if id < self.breakpoints.len() {
            self.breakpoints[id].2 = enabled;
            true
        } else {
            false
        }
    }
    
    pub fn get_breakpoints(&self) -> &[(String, u32, bool)] {
        &self.breakpoints
    }
    
    pub fn add_watch(&mut self, expression: &str) -> u32 {
        let id = self.watches.len() as u32;
        self.watches.push((expression.to_string(), id));
        id
    }
    
    pub fn remove_watch(&mut self, id: usize) -> bool {
        if id < self.watches.len() {
            self.watches.remove(id);
            true
        } else {
            false
        }
    }
    
    pub fn get_watches(&self) -> &[(String, u32)] {
        &self.watches
    }
    
    pub fn get_variables(&self) -> Vec<&Variable> {
        self.variables.values().collect()
    }
    
    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }
    
    pub fn get_frames(&self) -> &[(String, String, u32)] {
        &self.frames
    }
    
    pub fn get_current_frame(&self) -> Option<&(String, String, u32)> {
        self.frames.get(self.current_frame)
    }
    
    pub fn set_current_frame(&mut self, frame: usize) -> bool {
        if frame < self.frames.len() {
            self.current_frame = frame;
            true
        } else {
            false
        }
    }
    
    pub fn update_execution_point(&mut self, file: &str, line: u32, _column: u32, function: &str) {
        // Simulate adding a new frame
        if self.frames.is_empty() || self.frames[self.current_frame].0 != function {
            self.frames.push((function.to_string(), file.to_string(), line));
            self.current_frame = self.frames.len() - 1;
        } else {
            self.frames[self.current_frame] = (function.to_string(), file.to_string(), line);
        }
        
        // Simulate variables
        self.variables.clear();
        
        // Add some fake variables for demo purposes
        let string_val = VariableValue::String("Hello, World!".to_string());
        let int_val = VariableValue::Integer(42);
        let float_val = VariableValue::Float(3.14);
        let bool_val = VariableValue::Boolean(true);
        
        self.variables.insert("message".to_string(), 
            Variable { name: "message".to_string(), type_name: "String".to_string(), value: string_val, scope_level: 0, is_mutable: false });
        
        self.variables.insert("count".to_string(), 
            Variable { name: "count".to_string(), type_name: "i32".to_string(), value: int_val, scope_level: 0, is_mutable: true });
            
        self.variables.insert("pi".to_string(), 
            Variable { name: "pi".to_string(), type_name: "f64".to_string(), value: float_val, scope_level: 0, is_mutable: false });
            
        self.variables.insert("enabled".to_string(), 
            Variable { name: "enabled".to_string(), type_name: "bool".to_string(), value: bool_val, scope_level: 0, is_mutable: true });
    }
    
    pub fn continue_execution(&mut self, _flow_control: FlowControl) {
        // In a real implementation, this would resume the debugged program with the given flow control
        println!("Continuing execution...");
    }
    
    pub fn update_watches(&mut self) {
        // In a real implementation, this would update the watch expressions
        println!("Updating watches...");
    }
}

/// The current state of the debugger CLI
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// The current state of the debugger
    state: DebuggerState,
    /// The runtime engine
    runtime: DebuggerRuntime,
    /// The current source file being displayed
    current_file: Option<String>,
    /// The source file cache
    source_cache: HashMap<String, Vec<String>>,
    /// Standard output stream for colored output
    stdout: StandardStream,
}

impl Default for DebuggerCli {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerCli {
    /// Create a new DebuggerCli instance
    pub fn new() -> Self {
        Self {
            state: DebuggerState::Inactive,
            runtime: DebuggerRuntime::new(),
            current_file: None,
            source_cache: HashMap::new(),
            stdout: StandardStream::stdout(ColorChoice::Auto),
        }
    }
    
    /// Start the debugger CLI
    pub fn start(&mut self) {
        self.state = DebuggerState::Active;
        self.runtime.start();
        println!("Dbug debugger started. Type 'help' for a list of commands.");
    }
    
    /// Process a command from the user
    pub fn process_command(&mut self, command_line: &str) {
        let command_line = command_line.trim();
        if command_line.is_empty() {
            return;
        }
        
        // Split the command and arguments
        let parts: Vec<&str> = command_line.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];
        
        match command {
            "help" | "h" => self.print_help(),
            "quit" | "q" | "exit" => {
                println!("Exiting debugger");
                self.state = DebuggerState::Inactive;
                self.runtime.stop();
            },
            "next" | "n" => self.next(),
            "step" | "s" => self.step(),
            "continue" | "c" => self.continue_execution(),
            "break" | "b" => self.add_breakpoint(args),
            "delete" | "d" => self.delete_breakpoint(args),
            "enable" | "en" => self.toggle_breakpoint(args, true),
            "disable" | "dis" => self.toggle_breakpoint(args, false),
            "info" | "i" => self.show_info(args),
            "print" | "p" => self.print_variable(args),
            "watch" | "w" => self.add_watch(args),
            "unwatch" | "uw" => self.remove_watch(args),
            "list" | "l" => self.list_source(args),
            "backtrace" | "bt" => self.show_backtrace(),
            "frame" | "f" => self.select_frame(args),
            _ => println!("Unknown command: {}. Type 'help' for a list of commands.", command),
        }
    }
    
    /// Print help information
    fn print_help(&self) {
        println!("Dbug Debugger Commands:");
        println!("  n, next                   Step to the next line");
        println!("  s, step                   Step into a function call");
        println!("  c, continue               Continue execution until the next breakpoint");
        println!("  b, break [file:]line      Set a breakpoint");
        println!("  d, delete <num>           Delete a breakpoint");
        println!("  en, enable <num>          Enable a breakpoint");
        println!("  dis, disable <num>        Disable a breakpoint");
        println!("  i, info <type>            Show info about breakpoints, watches, etc.");
        println!("  p, print <expr>           Print the value of an expression");
        println!("  w, watch <expr>           Watch an expression for changes");
        println!("  uw, unwatch <num>         Remove a watch expression");
        println!("  l, list [file[:line]]     List source code");
        println!("  bt, backtrace             Show the call stack");
        println!("  f, frame <num>            Select a stack frame");
        println!("  q, quit                   Quit the debugger");
        println!("  help                      Show this help message");
    }
    
    /// Step over to the next line
    fn next(&mut self) {
        if self.state == DebuggerState::AtBreakpoint {
            println!("Stepping over...");
            self.runtime.continue_execution(FlowControl::StepOver);
            self.state = DebuggerState::Running;
            // In a real implementation, we would wait for the program to hit the next line
            // For now, just simulate stopping at another point
            self.simulate_breakpoint();
        } else {
            println!("Program not paused at a breakpoint. Use 'continue' to start execution.");
        }
    }
    
    /// Step into a function call
    fn step(&mut self) {
        if self.state == DebuggerState::AtBreakpoint {
            println!("Stepping into...");
            self.runtime.continue_execution(FlowControl::StepInto);
            self.state = DebuggerState::Running;
            // In a real implementation, we would wait for the program to hit the next line
            // For now, just simulate stopping at another point
            self.simulate_breakpoint();
        } else {
            println!("Program not paused at a breakpoint. Use 'continue' to start execution.");
        }
    }
    
    /// Continue execution until the next breakpoint
    fn continue_execution(&mut self) {
        if self.state == DebuggerState::AtBreakpoint {
            println!("Continuing...");
            self.runtime.continue_execution(FlowControl::Continue);
            self.state = DebuggerState::Running;
            // In a real implementation, we would wait for the program to hit the next breakpoint
            // For now, just simulate stopping at another breakpoint
            self.simulate_breakpoint();
        } else if self.state == DebuggerState::Active {
            println!("Starting program execution...");
            self.runtime.start();
            self.state = DebuggerState::Running;
            // In a real implementation, we would wait for the program to hit a breakpoint
            // For now, just simulate stopping at a breakpoint
            self.simulate_breakpoint();
        } else {
            println!("Program is already running.");
        }
    }
    
    /// Add a breakpoint
    fn add_breakpoint(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: break [file:]line");
            return;
        }
        
        let location = args[0];
        
        // Check if the location contains a file name
        if location.contains(':') {
            let parts: Vec<&str> = location.split(':').collect();
            if parts.len() == 2 {
                let file = parts[0];
                if let Ok(line) = parts[1].parse::<u32>() {
                    self.runtime.add_breakpoint(file, line, 0);
                } else {
                    println!("Invalid line number: {}", parts[1]);
                }
            } else {
                println!("Invalid breakpoint location: {}", location);
            }
        } else {
            // Try to parse as a line number
            if let Ok(line) = location.parse::<u32>() {
                if let Some(file) = &self.current_file {
                    self.runtime.add_breakpoint(file, line, 0);
                } else {
                    println!("No current file. Please specify a file:line location.");
                }
            } else {
                println!("Invalid line number: {}", location);
            }
        }
    }
    
    /// Delete a breakpoint
    fn delete_breakpoint(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: delete <breakpoint-id>");
            return;
        }
        
        if let Ok(id) = args[0].parse::<u32>() {
            self.runtime.remove_breakpoint(id);
        } else {
            println!("Invalid breakpoint id: {}", args[0]);
        }
    }
    
    /// Enable or disable a breakpoint
    fn toggle_breakpoint(&mut self, args: &[&str], enable: bool) {
        if args.is_empty() {
            println!("Usage: {} <breakpoint-id>", if enable { "enable" } else { "disable" });
            return;
        }
        
        if let Ok(id) = args[0].parse::<u32>() {
            self.runtime.toggle_breakpoint(id, enable);
        } else {
            println!("Invalid breakpoint id: {}", args[0]);
        }
    }
    
    /// Show information about debugger state
    fn show_info(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: info <breakpoints|watches|variables|frame>");
            return;
        }
        
        match args[0] {
            "breakpoints" | "b" => self.list_breakpoints(),
            "watches" | "w" => self.list_watches(),
            "variables" | "v" => self.list_variables(),
            "frame" | "f" => self.show_frame_info(),
            _ => println!("Unknown info type: {}", args[0]),
        }
    }
    
    /// List all breakpoints
    fn list_breakpoints(&self) {
        let breakpoints = self.runtime.list_breakpoints();
        if breakpoints.is_empty() {
            println!("No breakpoints set.");
            return;
        }
        
        println!("Breakpoints:");
        for bp in breakpoints {
            let status = if bp.enabled { "enabled" } else { "disabled" };
            let condition = match &bp.condition {
                Some(cond) => format!(" when {}", cond),
                None => String::new(),
            };
            println!("  #{}: {}:{}:{} ({}, hit {} times){}",
                     bp.id, bp.file, bp.line, bp.column, status, bp.hit_count, condition);
        }
    }
    
    /// List all watches
    fn list_watches(&self) {
        let watches = self.runtime.list_watches();
        if watches.is_empty() {
            println!("No watch expressions set.");
            return;
        }
        
        println!("Watch expressions:");
        for watch in watches {
            let status = if watch.enabled { "enabled" } else { "disabled" };
            let value = match &watch.last_value {
                Some(val) => val,
                None => "[not evaluated yet]",
            };
            println!("  #{}: {} ({}) = {}", watch.id, watch.expression, status, value);
        }
    }
    
    /// List all variables in the current scope
    fn list_variables(&self) {
        let variables = self.runtime.get_variables();
        if variables.is_empty() {
            println!("No variables in the current scope.");
            return;
        }
        
        println!("Variables in the current scope:");
        for var in variables {
            let mutability = if var.is_mutable { "mut " } else { "" };
            println!("  {}{}: {} = {}", mutability, var.name, var.type_name, var.value);
        }
    }
    
    /// Show information about the current frame
    fn show_frame_info(&self) {
        if let Some(point) = self.runtime.get_current_point() {
            println!("Current frame: {} at {}:{}:{}", point.function, point.file, point.line, point.column);
        } else {
            println!("No current frame (not at a breakpoint).");
        }
    }
    
    /// Print the value of a variable or expression
    fn print_variable(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: print <variable-name>");
            return;
        }
        
        let var_name = args[0];
        if let Some(variable) = self.runtime.variable_inspector.get_variable(var_name) {
            println!("{} = {}", var_name, variable.value);
        } else {
            println!("Variable '{}' not found in the current scope.", var_name);
        }
    }
    
    /// Add a watch expression
    fn add_watch(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: watch <expression>");
            return;
        }
        
        let expression = args.join(" ");
        self.runtime.add_watch(&expression);
    }
    
    /// Remove a watch expression
    fn remove_watch(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: unwatch <watch-id>");
            return;
        }
        
        if let Ok(id) = args[0].parse::<u32>() {
            self.runtime.remove_watch(id);
        } else {
            println!("Invalid watch id: {}", args[0]);
        }
    }
    
    /// List source code
    fn list_source(&mut self, args: &[&str]) {
        let (file, line) = if args.is_empty() {
            // Use current file and line if available
            if let Some(point) = self.runtime.get_current_point() {
                (point.file.clone(), point.line)
            } else if let Some(file) = &self.current_file {
                (file.clone(), 1)
            } else {
                println!("No current file. Please specify a file[:line] location.");
                return;
            }
        } else {
            // Parse the argument
            let arg = args[0];
            if arg.contains(':') {
                let parts: Vec<&str> = arg.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(line) = parts[1].parse::<u32>() {
                        (parts[0].to_string(), line)
                    } else {
                        println!("Invalid line number: {}", parts[1]);
                        return;
                    }
                } else {
                    println!("Invalid location: {}", arg);
                    return;
                }
            } else {
                // Try to parse as a line number for the current file
                if let Ok(line) = arg.parse::<u32>() {
                    if let Some(file) = &self.current_file {
                        (file.clone(), line)
                    } else {
                        println!("No current file. Please specify a file:line location.");
                        return;
                    }
                } else {
                    // Treat as a file name
                    (arg.to_string(), 1)
                }
            }
        };
        
        // Load the source file if needed
        if !self.source_cache.contains_key(&file) {
            if let Err(err) = self.load_source_file(&file) {
                println!("Error loading source file '{}': {}", file, err);
                return;
            }
        }
        
        // Update the current file
        self.current_file = Some(file.to_string());
        
        // Get the source lines
        let lines = self.source_cache.get(&file).unwrap();
        
        // Calculate the range to show
        let start_line = line.saturating_sub(5).max(1) as usize;
        let end_line = (line + 5).min(lines.len() as u32) as usize;
        
        // Display the source code
        println!("File: {}:{}:{}", file, line, 0);
        
        for i in start_line..=end_line {
            let line_num = i as u32;
            let is_current = line_num == line;
            let breakpoint = self.runtime.get_breakpoint_at(&file, line_num);
            
            // Format the line number
            let prefix = if is_current {
                "> "
            } else if breakpoint.is_some() {
                "B "
            } else {
                "  "
            };
            
            let line_text = if i <= lines.len() {
                &lines[i - 1]
            } else {
                "[end of file]"
            };
            
            // Print the line with appropriate coloring
            let stdout = &mut self.stdout;
            let _ = stdout.reset();
            
            if is_current {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_intense(true));
            } else if breakpoint.is_some() {
                let _ = stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_intense(true));
            }
            
            let _ = write!(stdout, "{}{:4} | ", prefix, line_num);
            let _ = stdout.reset();
            let _ = writeln!(stdout, "{}", line_text);
        }
        
        // Reset colors
        let _ = self.stdout.reset();
    }
    
    /// Load a source file into the cache
    fn load_source_file(&mut self, file: &str) -> io::Result<()> {
        let content = fs::read_to_string(file)?;
        let lines: Vec<String> = content.lines().map(String::from).collect();
        self.source_cache.insert(file.to_string(), lines);
        Ok(())
    }
    
    /// Show the backtrace
    fn show_backtrace(&self) {
        let frames = self.runtime.flow_controller.get_call_stack().get_frames();
        if frames.is_empty() {
            println!("No stack frames (not at a breakpoint).");
            return;
        }
        
        println!("Backtrace:");
        for (i, frame) in frames.iter().rev().enumerate() {
            println!("  #{} {} at {}:{}", i, frame.function, frame.file, frame.line);
        }
    }
    
    /// Select a stack frame
    fn select_frame(&mut self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: frame <frame-number>");
            return;
        }
        
        if let Ok(_frame_num) = args[0].parse::<usize>() {
            // In a real implementation, we would switch to the selected frame
            // For now, just print a message
            println!("Switching to frame {} (not implemented yet)", args[0]);
        } else {
            println!("Invalid frame number: {}", args[0]);
        }
    }
    
    /// Simulate stopping at a breakpoint (for demonstration purposes)
    fn simulate_breakpoint(&mut self) {
        // For demonstration purposes, create a sample execution point
        let file = "src/main.rs".to_string();
        let line = 42;
        let column = 0;
        let function = "main".to_string();
        
        // Update the runtime state
        self.runtime.update_execution_point(&file, line, column, &function);
        
        // Create some sample variables
        let var1 = Variable::new("x", "i32", VariableValue::Integer(42), 0, false);
        let var2 = Variable::new("y", "String", VariableValue::String("Hello, World!".to_string()), 0, true);
        
        // Register the variables
        self.runtime.register_variable(var1);
        self.runtime.register_variable(var2);
        
        // Update the CLI state
        self.state = DebuggerState::AtBreakpoint;
        
        // Load the source file (if it exists)
        if Path::new(&file).exists() {
            if let Err(err) = self.load_source_file(&file) {
                println!("Warning: Could not load source file '{}': {}", file, err);
            }
        }
        
        // Show the current location
        println!("Stopped at {}:{}:{} in {}", file, line, column, function);
        
        // List the source code
        self.list_source(&[]);
        
        // Update watches
        self.runtime.update_watches();
    }
} 