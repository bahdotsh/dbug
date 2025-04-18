// Runtime engine module for the dbug debugger

pub mod variables;
pub mod flow_control;

pub use variables::{Variable, VariableValue, VariableInspector};
pub use flow_control::{ExecutionState, FlowControl, ExecutionPoint, FlowController};

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
    #[allow(dead_code)]
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
    
    /// Register a hit of the breakpoint
    #[allow(dead_code)]
    pub fn register_hit(&mut self) {
        self.hit_count += 1;
    }
    
    /// Check if the breakpoint should trigger based on its condition
    #[allow(dead_code)]
    pub fn should_trigger(&self, variables: &VariableInspector) -> bool {
        if !self.enabled {
            return false;
        }
        
        // If there's no condition, always trigger
        if self.condition.is_none() {
            return true;
        }
        
        // Evaluate the condition based on the variables
        if let Some(condition) = &self.condition {
            // Simple expression evaluator for conditions
            // This is a basic implementation that supports:
            // - Variable comparisons (==, !=, <, >, <=, >=)
            // - Logical operators (&&, ||)
            
            // For complex conditions, we'll split by logical operators first
            if condition.contains("&&") {
                let parts: Vec<&str> = condition.split("&&").collect();
                return parts.iter().all(|part| self.evaluate_simple_condition(part.trim(), variables));
            } else if condition.contains("||") {
                let parts: Vec<&str> = condition.split("||").collect();
                return parts.iter().any(|part| self.evaluate_simple_condition(part.trim(), variables));
            } else {
                return self.evaluate_simple_condition(condition, variables);
            }
        }
        
        // Default to true if evaluation fails
        true
    }
    
    /// Evaluate a simple condition (without logical operators)
    #[allow(dead_code)]
    fn evaluate_simple_condition(&self, condition: &str, variables: &VariableInspector) -> bool {
        // Check for different comparison operators
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), "==", variables);
            }
        } else if condition.contains("!=") {
            let parts: Vec<&str> = condition.split("!=").collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), "!=", variables);
            }
        } else if condition.contains("<=") {
            let parts: Vec<&str> = condition.split("<=").collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), "<=", variables);
            }
        } else if condition.contains(">=") {
            let parts: Vec<&str> = condition.split(">=").collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), ">=", variables);
            }
        } else if condition.contains("<") {
            let parts: Vec<&str> = condition.split('<').collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), "<", variables);
            }
        } else if condition.contains(">") {
            let parts: Vec<&str> = condition.split('>').collect();
            if parts.len() == 2 {
                return self.compare_values(parts[0].trim(), parts[1].trim(), ">", variables);
            }
        }
        
        // If the condition is just a variable name, check if it's truthy
        if let Some(var) = variables.get_variable(condition) {
            return self.is_truthy(&var.value);
        }
        
        false
    }
    
    /// Compare two values based on the given operator
    #[allow(dead_code)]
    fn compare_values(&self, left: &str, right: &str, op: &str, variables: &VariableInspector) -> bool {
        // Get the left value (either a variable or a literal)
        let left_value = if let Some(var) = variables.get_variable(left) {
            Some(var.value.clone())
        } else {
            self.parse_literal(left)
        };
        
        // Get the right value (either a variable or a literal)
        let right_value = if let Some(var) = variables.get_variable(right) {
            Some(var.value.clone())
        } else {
            self.parse_literal(right)
        };
        
        // Compare the values based on the operator
        match (left_value, right_value) {
            (Some(left_val), Some(right_val)) => {
                self.compare_variable_values(&left_val, &right_val, op)
            }
            _ => false,
        }
    }
    
    /// Parse a literal value
    #[allow(dead_code)]
    fn parse_literal(&self, literal: &str) -> Option<VariableValue> {
        // Try to parse as integer
        if let Ok(i) = literal.parse::<i64>() {
            return Some(VariableValue::Integer(i));
        }
        
        // Try to parse as float
        if let Ok(f) = literal.parse::<f64>() {
            return Some(VariableValue::Float(f));
        }
        
        // Check for boolean literals
        if literal == "true" {
            return Some(VariableValue::Boolean(true));
        } else if literal == "false" {
            return Some(VariableValue::Boolean(false));
        }
        
        // Check for string literals
        if literal.starts_with('"') && literal.ends_with('"') && literal.len() >= 2 {
            return Some(VariableValue::String(literal[1..literal.len()-1].to_string()));
        }
        
        // Check for char literals
        if literal.starts_with('\'') && literal.ends_with('\'') && literal.len() == 3 {
            return Some(VariableValue::Char(literal.chars().nth(1).unwrap()));
        }
        
        None
    }
    
    /// Compare two variable values based on the given operator
    #[allow(dead_code)]
    fn compare_variable_values(&self, left: &VariableValue, right: &VariableValue, op: &str) -> bool {
        match (left, right) {
            (VariableValue::Integer(left_int), VariableValue::Integer(right_int)) => {
                match op {
                    "==" => left_int == right_int,
                    "!=" => left_int != right_int,
                    "<" => left_int < right_int,
                    ">" => left_int > right_int,
                    "<=" => left_int <= right_int,
                    ">=" => left_int >= right_int,
                    _ => false,
                }
            }
            (VariableValue::Float(left_float), VariableValue::Float(right_float)) => {
                match op {
                    "==" => (left_float - right_float).abs() < f64::EPSILON,
                    "!=" => (left_float - right_float).abs() >= f64::EPSILON,
                    "<" => left_float < right_float,
                    ">" => left_float > right_float,
                    "<=" => left_float <= right_float,
                    ">=" => left_float >= right_float,
                    _ => false,
                }
            }
            (VariableValue::Boolean(left_bool), VariableValue::Boolean(right_bool)) => {
                match op {
                    "==" => left_bool == right_bool,
                    "!=" => left_bool != right_bool,
                    _ => false,
                }
            }
            (VariableValue::String(left_str), VariableValue::String(right_str)) => {
                match op {
                    "==" => left_str == right_str,
                    "!=" => left_str != right_str,
                    "<" => left_str < right_str,
                    ">" => left_str > right_str,
                    "<=" => left_str <= right_str,
                    ">=" => left_str >= right_str,
                    _ => false,
                }
            }
            (VariableValue::Char(left_char), VariableValue::Char(right_char)) => {
                match op {
                    "==" => left_char == right_char,
                    "!=" => left_char != right_char,
                    "<" => left_char < right_char,
                    ">" => left_char > right_char,
                    "<=" => left_char <= right_char,
                    ">=" => left_char >= right_char,
                    _ => false,
                }
            }
            // For other types, only equality/inequality makes sense
            _ => {
                match op {
                    "==" => format!("{:?}", left) == format!("{:?}", right),
                    "!=" => format!("{:?}", left) != format!("{:?}", right),
                    _ => false,
                }
            }
        }
    }
    
    /// Check if a variable value is "truthy"
    #[allow(dead_code)]
    fn is_truthy(&self, value: &VariableValue) -> bool {
        match value {
            VariableValue::Boolean(b) => *b,
            VariableValue::Integer(i) => *i != 0,
            VariableValue::Float(f) => *f != 0.0 && !f.is_nan(),
            VariableValue::String(s) => !s.is_empty(),
            VariableValue::Char(_) => true,
            VariableValue::Array(arr) => !arr.is_empty(),
            VariableValue::Struct(fields) => !fields.is_empty(),
            VariableValue::Option(opt) => opt.is_some(),
            VariableValue::Reference(_) => true,
            VariableValue::Null => false,
            VariableValue::Complex { children, fields, .. } => {
                !fields.is_empty() || children.as_ref().is_some_and(|c| !c.is_empty())
            },
            VariableValue::Vec { elements, .. } => !elements.is_empty(),
            VariableValue::HashMap { entries, .. } => !entries.is_empty(),
        }
    }

    /// Evaluate a member access expression for the breakpoint
    /// This allows condition expressions to access struct members like "person.name"
    #[allow(dead_code)]
    fn evaluate_member_access(&self, base_var: &Variable, members: &[&str], _variables: &VariableInspector) -> Option<String> {
        evaluate_member_access_helper(base_var, members)
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
    #[allow(dead_code)]
    pub fn update_value(&mut self, value: &str) {
        self.last_value = Some(value.to_string());
    }
    
    /// Evaluate the expression and return the result as a string
    pub fn evaluate(&mut self, variables: &VariableInspector) -> String {
        if !self.enabled {
            return "[Disabled]".to_string();
        }
        
        // Simple expression evaluator
        let result = self.evaluate_expression(&self.expression, variables);
        let value = result.unwrap_or("[Evaluation failed]".to_string());
        
        // Update the last value
        self.last_value = Some(value.clone());
        
        value
    }
    
    /// Evaluate an expression and return the result as a string
    fn evaluate_expression(&self, expression: &str, variables: &VariableInspector) -> Option<String> {
        // Check if this is a simple variable reference
        if let Some(var) = variables.get_variable(expression) {
            return Some(format!("{}", var.value));
        }
        
        // Check for member access (e.g. "person.name")
        if expression.contains('.') {
            let parts: Vec<&str> = expression.split('.').collect();
            if parts.len() >= 2 {
                let base_var = variables.get_variable(parts[0])?;
                return self.evaluate_member_access(base_var, &parts[1..], variables);
            }
        }
        
        // Check for array access (e.g. "array[0]")
        if expression.contains('[') && expression.contains(']') {
            let start_bracket = expression.find('[')?;
            let end_bracket = expression.find(']')?;
            
            if start_bracket < end_bracket {
                let var_name = &expression[0..start_bracket];
                let index_expr = &expression[start_bracket+1..end_bracket];
                
                // Try to parse index as an integer
                if let Ok(index) = index_expr.parse::<usize>() {
                    if let Some(var) = variables.get_variable(var_name) {
                        return self.evaluate_array_access(var, index);
                    }
                }
            }
        }
        
        // For simple expressions like "x + 1", we'll parse and evaluate
        if let Some(result) = self.evaluate_arithmetic_expression(expression, variables) {
            return Some(result);
        }
        
        None
    }
    
    /// Evaluate member access expressions like "person.name"
    fn evaluate_member_access(&self, base_var: &Variable, members: &[&str], _variables: &VariableInspector) -> Option<String> {
        evaluate_member_access_helper(base_var, members)
    }
    
    /// Evaluate array access expressions like "array[0]"
    fn evaluate_array_access(&self, var: &Variable, index: usize) -> Option<String> {
        match &var.value {
            VariableValue::Array(array) => {
                if index < array.len() {
                    Some(format!("{}", array[index]))
                } else {
                    Some(format!("[Index {} out of bounds (len: {})]", index, array.len()))
                }
            }
            // For other types, we don't support array access
            _ => None,
        }
    }
    
    /// Evaluate arithmetic expressions like "x + 1"
    fn evaluate_arithmetic_expression(&self, expression: &str, variables: &VariableInspector) -> Option<String> {
        // Look for common operators: +, -, *, /, %
        for op in &["+", "-", "*", "/", "%"] {
            if expression.contains(op) {
                let parts: Vec<&str> = expression.split(op).collect();
                if parts.len() == 2 {
                    let left = parts[0].trim();
                    let right = parts[1].trim();
                    
                    // Get left value
                    let left_value = if let Some(var) = variables.get_variable(left) {
                        self.get_numeric_value(&var.value)
                    } else if let Ok(val) = left.parse::<f64>() {
                        Some(val)
                    } else {
                        None
                    };
                    
                    // Get right value
                    let right_value = if let Some(var) = variables.get_variable(right) {
                        self.get_numeric_value(&var.value)
                    } else if let Ok(val) = right.parse::<f64>() {
                        Some(val)
                    } else {
                        None
                    };
                    
                    // Perform the operation
                    if let (Some(left_val), Some(right_val)) = (left_value, right_value) {
                        let result = match *op {
                            "+" => left_val + right_val,
                            "-" => left_val - right_val,
                            "*" => left_val * right_val,
                            "/" => {
                                if right_val != 0.0 {
                                    left_val / right_val
                                } else {
                                    return Some("[Division by zero]".to_string());
                                }
                            }
                            "%" => {
                                if right_val != 0.0 {
                                    left_val % right_val
                                } else {
                                    return Some("[Modulo by zero]".to_string());
                                }
                            }
                            _ => return None, // Shouldn't happen
                        };
                        
                        // Format based on whether the result is a whole number
                        if result == (result as i64) as f64 {
                            return Some(format!("{}", result as i64));
                        } else {
                            return Some(format!("{}", result));
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Get numeric value from a variable
    fn get_numeric_value(&self, value: &VariableValue) -> Option<f64> {
        match value {
            VariableValue::Integer(i) => Some(*i as f64),
            VariableValue::Float(f) => Some(*f),
            VariableValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            VariableValue::Char(c) => Some(*c as u32 as f64),
            _ => None,
        }
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn should_break_at(&mut self, file: &str, line: u32, _column: u32) -> bool {
        for breakpoint in &mut self.breakpoints {
            if breakpoint.file == file && breakpoint.line == line && breakpoint.enabled && breakpoint.should_trigger(&self.variable_inspector) {
                breakpoint.register_hit();
                return true;
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
                // Evaluate the expression and update the value
                let value = watch.evaluate(&self.variable_inspector);
                println!("Watch #{}: {} = {}", watch.id, watch.expression, value);
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
    
    /// Visualize a variable in detail, including complex data structures
    #[allow(dead_code)]
    pub fn visualize_variable(&self, name: &str) -> Option<String> {
        self.variable_inspector.visualize_variable(name)
    }
}

impl Default for DebuggerRuntime {
    fn default() -> Self {
        Self::new()
    }
} 