// Runtime engine module for the dbug debugger

mod variables;
mod flow_control;

pub use variables::{Variable, VariableValue, VariableInspector};
pub use flow_control::{ExecutionState, FlowControl, ExecutionPoint, StackFrame, CallStack, FlowController};

/// A breakpoint in the code
#[derive(Debug, Clone)]
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
    /// The hit count of the breakpoint
    pub hit_count: u32,
    /// The id of the breakpoint
    pub id: u32,
}

impl Breakpoint {
    /// Create a new breakpoint
    pub fn new(file: &str, line: u32, column: u32, id: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            enabled: true,
            condition: None,
            hit_count: 0,
            id,
        }
    }
    
    /// Set a condition for the breakpoint
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
    
    /// Register a hit of the breakpoint
    pub fn register_hit(&mut self) {
        self.hit_count += 1;
    }
    
    /// Check if the breakpoint should trigger based on its condition
    pub fn should_trigger(&self, variables: &VariableInspector) -> bool {
        if !self.enabled {
            return false;
        }
        
        // If there's no condition, always trigger
        if self.condition.is_none() {
            return true;
        }
        
        // TODO: Evaluate the condition based on the variables
        // For now, always trigger if there's a condition
        true
    }
}

/// A watch expression
#[derive(Debug, Clone)]
pub struct WatchExpression {
    /// The expression to watch
    pub expression: String,
    /// The last value of the expression
    pub last_value: Option<String>,
    /// Whether the watch is enabled
    pub enabled: bool,
    /// The id of the watch
    pub id: u32,
}

impl WatchExpression {
    /// Create a new watch expression
    pub fn new(expression: &str, id: u32) -> Self {
        Self {
            expression: expression.to_string(),
            last_value: None,
            enabled: true,
            id,
        }
    }
    
    /// Update the value of the watch expression
    pub fn update_value(&mut self, value: &str) {
        self.last_value = Some(value.to_string());
    }
}

/// The runtime engine for the debugger
pub struct DebuggerRuntime {
    /// The breakpoints that have been set
    pub breakpoints: Vec<Breakpoint>,
    /// The watch expressions that have been set
    pub watches: Vec<WatchExpression>,
    /// Variable inspector for managing variables
    pub variable_inspector: VariableInspector,
    /// Flow controller for managing execution
    pub flow_controller: FlowController,
    /// The next breakpoint id to assign
    next_breakpoint_id: u32,
    /// The next watch id to assign
    next_watch_id: u32,
}

impl DebuggerRuntime {
    /// Create a new DebuggerRuntime
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            watches: Vec::new(),
            variable_inspector: VariableInspector::new(),
            flow_controller: FlowController::new(),
            next_breakpoint_id: 1,
            next_watch_id: 1,
        }
    }
    
    /// Start the debugger runtime
    pub fn start(&mut self) {
        self.flow_controller.start();
        println!("Debugger runtime started");
    }
    
    /// Stop the debugger runtime
    pub fn stop(&mut self) {
        self.flow_controller.stop();
        println!("Debugger runtime stopped");
    }
    
    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, file: &str, line: u32, column: u32) -> u32 {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        let breakpoint = Breakpoint::new(file, line, column, id);
        println!("Added breakpoint #{} at {}:{}:{}", id, file, line, column);
        self.breakpoints.push(breakpoint);
        id
    }
    
    /// Add a conditional breakpoint
    pub fn add_conditional_breakpoint(&mut self, file: &str, line: u32, column: u32, condition: &str) -> u32 {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        let breakpoint = Breakpoint::new(file, line, column, id).with_condition(condition);
        println!("Added conditional breakpoint #{} at {}:{}:{} when {}", 
                id, file, line, column, condition);
        self.breakpoints.push(breakpoint);
        id
    }
    
    /// Remove a breakpoint by id
    pub fn remove_breakpoint(&mut self, id: u32) -> bool {
        let len_before = self.breakpoints.len();
        self.breakpoints.retain(|b| b.id != id);
        let removed = len_before > self.breakpoints.len();
        if removed {
            println!("Removed breakpoint #{}", id);
        } else {
            println!("Breakpoint #{} not found", id);
        }
        removed
    }
    
    /// Enable or disable a breakpoint
    pub fn toggle_breakpoint(&mut self, id: u32, enabled: bool) -> bool {
        for breakpoint in &mut self.breakpoints {
            if breakpoint.id == id {
                breakpoint.enabled = enabled;
                println!("{} breakpoint #{}", if enabled { "Enabled" } else { "Disabled" }, id);
                return true;
            }
        }
        println!("Breakpoint #{} not found", id);
        false
    }
    
    /// Add a watch expression
    pub fn add_watch(&mut self, expression: &str) -> u32 {
        let id = self.next_watch_id;
        self.next_watch_id += 1;
        
        let watch = WatchExpression::new(expression, id);
        println!("Added watch #{} for '{}'", id, expression);
        self.watches.push(watch);
        id
    }
    
    /// Remove a watch expression by id
    pub fn remove_watch(&mut self, id: u32) -> bool {
        let len_before = self.watches.len();
        self.watches.retain(|w| w.id != id);
        let removed = len_before > self.watches.len();
        if removed {
            println!("Removed watch #{}", id);
        } else {
            println!("Watch #{} not found", id);
        }
        removed
    }
    
    /// Check if execution should break at the current point
    pub fn should_break_at(&mut self, file: &str, line: u32, column: u32) -> bool {
        for breakpoint in &mut self.breakpoints {
            if breakpoint.file == file && breakpoint.line == line && breakpoint.enabled {
                if breakpoint.should_trigger(&self.variable_inspector) {
                    breakpoint.register_hit();
                    return true;
                }
            }
        }
        false
    }
    
    /// Get the breakpoint at a specific location, if any
    pub fn get_breakpoint_at(&self, file: &str, line: u32) -> Option<&Breakpoint> {
        self.breakpoints.iter().find(|b| b.file == file && b.line == line)
    }
    
    /// Update the watch expressions
    pub fn update_watches(&mut self) {
        for watch in &mut self.watches {
            if watch.enabled {
                // TODO: Evaluate the expression and update the value
                // For now, set a placeholder value
                watch.update_value("[Value not implemented yet]");
            }
        }
    }
    
    /// Update the execution point
    pub fn update_execution_point(&mut self, file: &str, line: u32, column: u32, function: &str) {
        let stack_depth = self.flow_controller.get_call_stack().depth() as u32;
        let point = ExecutionPoint::new(file, line, column, function, stack_depth);
        self.flow_controller.update_execution_point(point);
    }
    
    /// Continue execution with the specified flow control
    pub fn continue_execution(&mut self, control: FlowControl) {
        self.flow_controller.resume(control);
    }
    
    /// Enter a function
    pub fn enter_function(&mut self, function: &str, file: &str, line: u32) {
        self.flow_controller.enter_function(function, file, line);
        self.variable_inspector.enter_scope();
    }
    
    /// Exit a function
    pub fn exit_function(&mut self) {
        self.flow_controller.exit_function();
        self.variable_inspector.exit_scope();
    }
    
    /// Register a variable
    pub fn register_variable(&mut self, variable: Variable) {
        self.variable_inspector.register_variable(variable);
    }
    
    /// Get the current execution state
    pub fn get_execution_state(&self) -> ExecutionState {
        self.flow_controller.get_state()
    }
    
    /// Get the current execution point
    pub fn get_current_point(&self) -> Option<&ExecutionPoint> {
        self.flow_controller.get_current_point()
    }
    
    /// List all breakpoints
    pub fn list_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }
    
    /// List all watch expressions
    pub fn list_watches(&self) -> &[WatchExpression] {
        &self.watches
    }
    
    /// Get all variables in the current scope
    pub fn get_variables(&self) -> Vec<&Variable> {
        self.variable_inspector.get_all_variables()
    }
} 