// Runtime engine module for the dbug debugger

pub mod variables;
pub mod flow_control;
pub mod type_visualization;

pub use variables::{Variable, VariableValue, VariableInspector, ChangeStatus};
pub use flow_control::{ExecutionState, FlowControl, ExecutionPoint, FlowController};
pub use type_visualization::TypeVisualizer;

use std::sync::atomic::{AtomicU8, Ordering};
use crate::errors::{DbugResult, DbugError};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

// Flow control constants
static FLOW_CONTROL: AtomicU8 = AtomicU8::new(0);

const FLOW_CONTINUE: u8 = 0;
const FLOW_STEP_OVER: u8 = 1;
const FLOW_STEP_INTO: u8 = 2;
const FLOW_STEP_OUT: u8 = 3;

/// Global runtime instance
static DEBUGGER_RUNTIME: Lazy<Arc<Mutex<DebuggerRuntime>>> = Lazy::new(|| {
    Arc::new(Mutex::new(DebuggerRuntime::new()))
});

/// Gets a reference to the global runtime
fn get_global_runtime() -> std::sync::MutexGuard<'static, DebuggerRuntime> {
    DEBUGGER_RUNTIME.lock().unwrap_or_else(|e| {
        panic!("Failed to lock global runtime: {}", e);
    })
}

/// Sets the flow control to continue execution
pub fn set_continue() -> DbugResult<()> {
    set_flow_control(FlowControl::Continue)
}

/// Sets the flow control to step over
pub fn set_step_over() -> DbugResult<()> {
    set_flow_control(FlowControl::StepOver)
}

/// Sets the flow control to step into
pub fn set_step_into() -> DbugResult<()> {
    set_flow_control(FlowControl::StepInto)
}

/// Sets the flow control to step out
pub fn set_step_out() -> DbugResult<()> {
    set_flow_control(FlowControl::StepOut)
}

/// Sets the flow control
pub fn set_flow_control(control: FlowControl) -> DbugResult<()> {
    let value = match control {
        FlowControl::Continue => FLOW_CONTINUE,
        FlowControl::StepOver => FLOW_STEP_OVER,
        FlowControl::StepInto => FLOW_STEP_INTO,
        FlowControl::StepOut => FLOW_STEP_OUT,
        _ => FLOW_CONTINUE, // Default to continue for other values
    };
    
    FLOW_CONTROL.store(value, Ordering::SeqCst);
    Ok(())
}

/// Gets the current flow control
pub fn get_flow_control() -> FlowControl {
    match FLOW_CONTROL.load(Ordering::SeqCst) {
        FLOW_STEP_OVER => FlowControl::StepOver,
        FLOW_STEP_INTO => FlowControl::StepInto,
        FLOW_STEP_OUT => FlowControl::StepOut,
        _ => FlowControl::Continue,
    }
}

/// Gets the current variables in scope
pub fn get_current_variables() -> DbugResult<VariableInspector> {
    // This is a simplified implementation that uses a global runtime
    // In a real system, this would fetch variables from the debugged process
    let runtime = get_global_runtime();
    Ok(runtime.variable_inspector.clone())
}

/// Evaluates an expression in the current context
pub fn evaluate_expression(expression: &str, variables: &VariableInspector) -> Option<String> {
    // Create a temporary watch and evaluate it
    let mut temp_watch = WatchExpression::new(expression, 0);
    Some(temp_watch.evaluate(variables))
}

/// Helper function for evaluating member access expressions
/// Split out to avoid clippy warnings about recursion parameters
fn evaluate_member_access_helper(base_var: &Variable, members: &[&str]) -> Option<String> {
    if members.is_empty() {
        return Some(base_var.value.to_string());
    }

    // Get the first member
    let member = members[0];

    // Handle the member access based on the base variable's type
    match &base_var.value {
        VariableValue::Struct(fields) => {
            if let Some(field_value) = fields.get(member) {
                if members.len() == 1 {
                    // This is the last member, return its value
                    return Some(field_value.to_string());
                } else {
                    // Create a temporary variable for the field
                    let temp_var = Variable::new(
                        member,
                        "field",
                        field_value.clone(),
                        base_var.scope_level,
                        false
                    );
                    
                    // Recursively evaluate the next member
                    return evaluate_member_access_helper(&temp_var, &members[1..]);
                }
            }
        }
        // For other types, we don't support member access
        _ => {}
    }

    None
}

/// Condition mode for breakpoints
#[derive(Debug, Clone, PartialEq)]
pub enum BreakpointConditionMode {
    /// Always break when hit
    Always,
    /// Break when the condition expression evaluates to true
    ConditionalExpression(String),
    /// Break when hit count meets the criteria
    HitCount(HitCountCondition),
    /// Break when both condition and hit count criteria are met
    Combined {
        /// The condition expression
        expression: String,
        /// The hit count condition
        hit_count: HitCountCondition,
    },
}

/// Hit count condition for breakpoints
#[derive(Debug, Clone, PartialEq)]
pub enum HitCountCondition {
    /// Break when hit count equals the target
    Equals(u32),
    /// Break when hit count is greater than the target
    GreaterThan(u32),
    /// Break when hit count is a multiple of the target
    Multiple(u32),
}

impl HitCountCondition {
    /// Parse a hit count condition from a string
    /// Format: "= N", "> N", "% N"
    pub fn from_string(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('=') {
            let count_str = s[1..].trim();
            count_str.parse::<u32>().ok().map(HitCountCondition::Equals)
        } else if s.starts_with('>') {
            let count_str = s[1..].trim();
            count_str.parse::<u32>().ok().map(HitCountCondition::GreaterThan)
        } else if s.starts_with('%') {
            let count_str = s[1..].trim();
            count_str.parse::<u32>().ok().map(HitCountCondition::Multiple)
        } else {
            // Default to equals if just a number is provided
            s.parse::<u32>().ok().map(HitCountCondition::Equals)
        }
    }
    
    /// Check if the hit count meets the condition
    pub fn is_met(&self, hit_count: u32) -> bool {
        match self {
            HitCountCondition::Equals(target) => hit_count == *target,
            HitCountCondition::GreaterThan(target) => hit_count > *target,
            HitCountCondition::Multiple(target) if *target > 0 => hit_count % *target == 0,
            _ => false,
        }
    }
}

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
    /// The condition for the breakpoint to trigger
    pub condition_mode: BreakpointConditionMode,
    /// The hit count of the breakpoint
    pub hit_count: u32,
    /// The id of the breakpoint
    pub id: u32,
    /// When this breakpoint was created
    pub created_at: std::time::Instant,
    /// When this breakpoint was last hit
    pub last_hit: Option<std::time::Instant>,
}

impl Breakpoint {
    /// Create a new breakpoint
    pub fn new(file: &str, line: u32, column: u32, id: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            enabled: true,
            condition_mode: BreakpointConditionMode::Always,
            hit_count: 0,
            id,
            created_at: std::time::Instant::now(),
            last_hit: None,
        }
    }
    
    /// Set a condition for the breakpoint
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition_mode = BreakpointConditionMode::ConditionalExpression(condition.to_string());
        self
    }
    
    /// Set a hit count condition for the breakpoint
    pub fn with_hit_count_condition(mut self, condition: HitCountCondition) -> Self {
        self.condition_mode = BreakpointConditionMode::HitCount(condition);
        self
    }
    
    /// Set both a condition and hit count for the breakpoint
    pub fn with_combined_condition(mut self, expression: &str, hit_count: HitCountCondition) -> Self {
        self.condition_mode = BreakpointConditionMode::Combined {
            expression: expression.to_string(),
            hit_count,
        };
        self
    }
    
    /// Register a hit of the breakpoint
    pub fn register_hit(&mut self) {
        self.hit_count += 1;
        self.last_hit = Some(std::time::Instant::now());
    }
    
    /// Check if the breakpoint should trigger based on its condition
    pub fn should_trigger(&self, variables: &VariableInspector) -> bool {
        if !self.enabled {
            return false;
        }
        
        match &self.condition_mode {
            BreakpointConditionMode::Always => true,
            
            BreakpointConditionMode::ConditionalExpression(expr) => {
                self.evaluate_condition(expr, variables)
            },
            
            BreakpointConditionMode::HitCount(condition) => {
                condition.is_met(self.hit_count)
            },
            
            BreakpointConditionMode::Combined { expression, hit_count } => {
                hit_count.is_met(self.hit_count) && self.evaluate_condition(expression, variables)
            },
        }
    }
    
    /// Evaluate a condition expression
    fn evaluate_condition(&self, condition: &str, variables: &VariableInspector) -> bool {
        // Create a temporary watch expression to evaluate the condition
        let mut temp_watch = WatchExpression::new(condition, 0);
        let result = temp_watch.evaluate(variables);
        
        // Try to convert the result to a boolean
        match result.to_lowercase().as_str() {
            "true" => true,
            "false" => false,
            // If the result is a number, treat 0 as false and non-zero as true
            _ => {
                if let Ok(num) = result.parse::<f64>() {
                    num != 0.0
                } else {
                    // If we can't parse as a boolean or number, default to false
                    false
                }
            }
        }
    }
    
    /// Check if this breakpoint is at a specific location
    pub fn is_at_location(&self, file: &str, line: u32) -> bool {
        self.file == file && self.line == line
    }
    
    /// Get time since creation
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
    
    /// Get time since last hit
    pub fn time_since_last_hit(&self) -> Option<std::time::Duration> {
        self.last_hit.map(|time| time.elapsed())
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
    /// Whether the value has changed since last evaluation
    pub has_changed: bool,
    /// Time of the last change
    pub last_change: Option<std::time::Instant>,
    /// Number of times the watch value has changed
    pub change_count: u32,
}

impl WatchExpression {
    /// Create a new watch expression
    pub fn new(expression: &str, id: u32) -> Self {
        Self {
            expression: expression.to_string(),
            last_value: None,
            enabled: true,
            id,
            has_changed: false,
            last_change: None,
            change_count: 0,
        }
    }
    
    /// Update the last value
    pub fn update_value(&mut self, value: &str) {
        let has_changed = match &self.last_value {
            Some(last) => last != value,
            None => true,
        };
        
        self.last_value = Some(value.to_string());
        
        if has_changed {
            self.has_changed = true;
            self.last_change = Some(std::time::Instant::now());
            self.change_count += 1;
        }
    }
    
    /// Evaluate the expression against the current variable state
    pub fn evaluate(&mut self, variables: &VariableInspector) -> String {
        if !self.enabled {
            return String::from("[Watch disabled]");
        }
        
        // Try to evaluate the expression
        let result = match self.evaluate_expression(&self.expression, variables) {
            Some(value) => value,
            None => String::from("[Evaluation failed]"),
        };
        
        // Update the last value and check for changes
        self.update_value(&result);
        
        result
    }
    
    /// Evaluate a complex expression
    fn evaluate_expression(&self, expression: &str, variables: &VariableInspector) -> Option<String> {
        // First, check if the expression is a simple variable name
        if let Some(var) = variables.get_variable(expression) {
            return Some(var.value.to_string());
        }
        
        // Check for member access (e.g., person.name)
        if expression.contains('.') {
            let parts: Vec<&str> = expression.split('.').collect();
            if parts.len() >= 2 {
                let base_var_name = parts[0];
                let members = &parts[1..];
                
                if let Some(base_var) = variables.get_variable(base_var_name) {
                    return evaluate_member_access_helper(base_var, members);
                }
            }
        }
        
        // Check for array/vector access (e.g., arr[0])
        if let Some(bracket_pos) = expression.find('[') {
            if expression.ends_with(']') {
                let var_name = &expression[..bracket_pos];
                let index_str = &expression[bracket_pos + 1..expression.len() - 1];
                
                if let Some(var) = variables.get_variable(var_name) {
                    if let Ok(index) = index_str.parse::<usize>() {
                        return self.evaluate_array_access(var, index);
                    }
                }
            }
        }
        
        // Try to evaluate arithmetic expressions using basic operators
        if expression.contains('+') || expression.contains('-') || 
           expression.contains('*') || expression.contains('/') {
            return self.evaluate_arithmetic_expression(expression, variables);
        }
        
        // Try more complex expressions (function calls, complex conditionals)
        self.evaluate_complex_expression(expression, variables)
    }
    
    /// Evaluate array access expressions
    fn evaluate_array_access(&self, var: &Variable, index: usize) -> Option<String> {
        match &var.value {
            VariableValue::Array(elements) => {
                if index < elements.len() {
                    return Some(elements[index].to_string());
                }
            },
            VariableValue::Vec { elements, .. } => {
                if index < elements.len() {
                    return Some(elements[index].to_string());
                }
            },
            _ => {}
        }
        
        None
    }
    
    /// Evaluate arithmetic expressions
    fn evaluate_arithmetic_expression(&self, expression: &str, variables: &VariableInspector) -> Option<String> {
        // Very simple arithmetic evaluation for basic operations
        // In a real implementation, this would use a proper expression parser
        
        // Find the operator
        let op_pos = expression.find(|c| c == '+' || c == '-' || c == '*' || c == '/');
        if let Some(pos) = op_pos {
            let left_expr = expression[..pos].trim();
            let right_expr = expression[pos + 1..].trim();
            let operator = expression.chars().nth(pos).unwrap();
            
            // Evaluate left and right operands recursively
            let left_result = self.evaluate_expression(left_expr, variables)
                .and_then(|val| val.parse::<f64>().ok());
            
            let right_result = self.evaluate_expression(right_expr, variables)
                .and_then(|val| val.parse::<f64>().ok());
            
            // Perform the operation
            if let (Some(left), Some(right)) = (left_result, right_result) {
                let result = match operator {
                    '+' => left + right,
                    '-' => left - right,
                    '*' => left * right,
                    '/' => {
                        if right == 0.0 {
                            return Some(String::from("[Division by zero]"));
                        }
                        left / right
                    },
                    _ => return None,
                };
                
                // Convert result to string
                return Some(result.to_string());
            }
        }
        
        None
    }
    
    /// Evaluate more complex expressions like function calls or conditionals
    fn evaluate_complex_expression(&self, expression: &str, variables: &VariableInspector) -> Option<String> {
        // Check for conditional expressions (e.g., x > y)
        for &op in &["==", "!=", ">=", "<=", ">", "<"] {
            if expression.contains(op) {
                return self.evaluate_conditional(expression, op, variables);
            }
        }
        
        // For now, we don't support function calls or more complex expressions
        None
    }
    
    /// Evaluate conditional expressions
    fn evaluate_conditional(&self, expression: &str, operator: &str, variables: &VariableInspector) -> Option<String> {
        let parts: Vec<&str> = expression.split(operator).collect();
        if parts.len() != 2 {
            return None;
        }
        
        let left = parts[0].trim();
        let right = parts[1].trim();
        
        // Evaluate both sides
        let left_value = self.evaluate_expression(left, variables);
        let right_value = self.evaluate_expression(right, variables);
        
        match (left_value, right_value) {
            (Some(left_str), Some(right_str)) => {
                // Try numeric comparison first
                if let (Ok(left_num), Ok(right_num)) = (left_str.parse::<f64>(), right_str.parse::<f64>()) {
                    let result = match operator {
                        "==" => left_num == right_num,
                        "!=" => left_num != right_num,
                        ">=" => left_num >= right_num,
                        "<=" => left_num <= right_num,
                        ">" => left_num > right_num,
                        "<" => left_num < right_num,
                        _ => return None,
                    };
                    return Some(result.to_string());
                }
                
                // Fallback to string comparison
                let result = match operator {
                    "==" => left_str == right_str,
                    "!=" => left_str != right_str,
                    _ => return None, // String doesn't support other comparisons
                };
                
                Some(result.to_string())
            },
            _ => None,
        }
    }
    
    /// Reset the change flag after it's been seen by the UI
    pub fn acknowledge_change(&mut self) {
        self.has_changed = false;
    }
    
    /// Check if the expression result has changed since last evaluation
    pub fn has_changed(&self) -> bool {
        self.has_changed
    }
    
    /// Get the time since last change
    pub fn time_since_change(&self) -> Option<std::time::Duration> {
        self.last_change.map(|time| time.elapsed())
    }
    
    /// Toggle the enabled state
    pub fn toggle(&mut self, enabled: bool) {
        self.enabled = enabled;
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
    /// Type visualizer for complex types
    pub type_visualizer: TypeVisualizer,
    /// The next breakpoint id to assign
    next_breakpoint_id: u32,
    /// The next watch id to assign
    next_watch_id: u32,
}

impl DebuggerRuntime {
    /// Create a new debugger runtime
    pub fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            watches: Vec::new(),
            variable_inspector: VariableInspector::new(),
            flow_controller: FlowController::new(),
            type_visualizer: TypeVisualizer::default(),
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
        
        // Create a new breakpoint with the condition
        let breakpoint = Breakpoint::new(file, line, column, id)
            .with_condition(condition);
        
        self.breakpoints.push(breakpoint);
        id
    }
    
    /// Add a hit count breakpoint
    pub fn add_hit_count_breakpoint(&mut self, file: &str, line: u32, column: u32, hit_count_expr: &str) -> DbugResult<u32> {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        // Parse the hit count expression
        let hit_count_condition = HitCountCondition::from_string(hit_count_expr)
            .ok_or_else(|| DbugError::CliError(format!("Invalid hit count expression: {}", hit_count_expr)))?;
        
        // Create a new breakpoint with the hit count condition
        let breakpoint = Breakpoint::new(file, line, column, id)
            .with_hit_count_condition(hit_count_condition);
        
        self.breakpoints.push(breakpoint);
        Ok(id)
    }
    
    /// Add a combined condition and hit count breakpoint
    pub fn add_combined_breakpoint(&mut self, file: &str, line: u32, column: u32, condition: &str, hit_count_expr: &str) -> DbugResult<u32> {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        // Parse the hit count expression
        let hit_count_condition = HitCountCondition::from_string(hit_count_expr)
            .ok_or_else(|| DbugError::CliError(format!("Invalid hit count expression: {}", hit_count_expr)))?;
        
        // Create a new breakpoint with both conditions
        let breakpoint = Breakpoint::new(file, line, column, id)
            .with_combined_condition(condition, hit_count_condition);
        
        self.breakpoints.push(breakpoint);
        Ok(id)
    }
    
    /// Remove a breakpoint by id
    pub fn remove_breakpoint(&mut self, id: u32) -> bool {
        let pos = self.breakpoints.iter().position(|b| b.id == id);
        if let Some(pos) = pos {
            self.breakpoints.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Toggle a breakpoint's enabled state
    pub fn toggle_breakpoint(&mut self, id: u32, enabled: bool) -> bool {
        if let Some(breakpoint) = self.breakpoints.iter_mut().find(|b| b.id == id) {
            breakpoint.enabled = enabled;
            true
        } else {
            false
        }
    }
    
    /// Find a breakpoint by file and line
    pub fn find_breakpoint(&self, file: &str, line: u32) -> Option<&Breakpoint> {
        self.breakpoints.iter().find(|b| b.is_at_location(file, line))
    }
    
    /// Find a breakpoint by id
    pub fn find_breakpoint_by_id(&self, id: u32) -> Option<&Breakpoint> {
        self.breakpoints.iter().find(|b| b.id == id)
    }
    
    /// Check if execution should break at a given location
    pub fn should_break_at(&mut self, file: &str, line: u32, _column: u32) -> bool {
        // Find any breakpoints at this location
        let matching_breakpoints: Vec<_> = self.breakpoints.iter_mut()
            .filter(|b| b.is_at_location(file, line))
            .collect();
        
        for breakpoint in matching_breakpoints {
            // Register a hit regardless of whether we actually break
            breakpoint.register_hit();
            
            // Check if we should trigger based on conditions
            if breakpoint.should_trigger(&self.variable_inspector) {
                return true;
            }
        }
        
        false
    }
    
    /// Remove all breakpoints
    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }
    
    /// Remove all disabled breakpoints
    pub fn clear_disabled_breakpoints(&mut self) {
        self.breakpoints.retain(|b| b.enabled);
    }
    
    /// Get all breakpoints
    pub fn list_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
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
    #[allow(dead_code)]
    pub fn enter_function(&mut self, function: &str, file: &str, line: u32) {
        self.flow_controller.enter_function(function, file, line);
        self.variable_inspector.enter_scope();
    }
    
    /// Exit a function
    #[allow(dead_code)]
    pub fn exit_function(&mut self) {
        self.flow_controller.exit_function();
        self.variable_inspector.exit_scope();
    }
    
    /// Register a variable
    pub fn register_variable(&mut self, variable: Variable) {
        self.variable_inspector.register_variable(variable);
    }
    
    /// Get the current execution state
    #[allow(dead_code)]
    pub fn get_execution_state(&self) -> ExecutionState {
        self.flow_controller.get_state()
    }
    
    /// Get the current execution point
    pub fn get_current_point(&self) -> Option<&ExecutionPoint> {
        self.flow_controller.get_current_point()
    }
    
    /// List all watch expressions
    pub fn list_watches(&self) -> &[WatchExpression] {
        &self.watches
    }
    
    /// Get all variables in the current scope
    pub fn get_variables(&self) -> Vec<&Variable> {
        self.variable_inspector.get_all_variables()
    }
    
    /// Visualize a variable with advanced type handling
    pub fn visualize_variable(&self, name: &str) -> Option<String> {
        // Get the variable from the inspector
        let variable = self.variable_inspector.get_variable(name)?;
        
        // Try to use a custom visualizer if available
        if self.type_visualizer.has_visualizer(&variable.type_name) {
            self.type_visualizer.visualize(variable)
        } else {
            // Fall back to default visualization
            Some(self.type_visualizer.create_composite_visualization(
                &variable.type_name, 
                variable,
                0
            ))
        }
    }
    
    /// Register a custom type visualizer
    pub fn register_type_visualizer<F>(&mut self, type_name: &str, visualizer: F)
    where
        F: Fn(&Variable) -> Option<String> + Send + Sync + 'static,
    {
        self.type_visualizer.register_visualizer(type_name, visualizer);
    }
    
    /// Get all changed variables since last check
    pub fn get_changed_variables(&mut self) -> Vec<&Variable> {
        self.variable_inspector.get_changed_variables()
    }
    
    /// Reset change tracking
    pub fn reset_change_tracking(&mut self) {
        self.variable_inspector.reset_change_status();
        
        // Reset change flags in watches
        for watch in &mut self.watches {
            watch.acknowledge_change();
        }
    }
}

impl Default for DebuggerRuntime {
    fn default() -> Self {
        Self::new()
    }
} 