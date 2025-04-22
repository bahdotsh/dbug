// Flow control functionality for the runtime debugger
use crate::errors::DbugResult;
use crate::runtime::variables::VariableValue;
use crate::runtime::Breakpoint;
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicU32;
use std::sync::RwLock;

/// The current execution state of the debugger
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// The program is not running
    NotRunning,
    /// The program is running
    Running,
    /// The program is paused at a breakpoint
    Paused,
    /// The program has completed execution
    Completed,
    /// The program has terminated with an error
    Error,
}

/// Instruction for how to continue execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    /// Continue execution until the next breakpoint
    Continue,
    /// Step to the next line, stepping over function calls
    StepOver,
    /// Step to the next line, stepping into function calls
    StepInto,
    /// Step out of the current function
    StepOut,
    /// Run to the cursor position
    RunToCursor,
    /// Stop execution
    Stop,
}

/// Information about the current execution point
#[derive(Debug, Clone)]
pub struct ExecutionPoint {
    /// The file being executed
    pub file: String,
    /// The line number
    pub line: u32,
    /// The column number
    pub column: u32,
    /// The function name
    pub function: String,
    /// The stack depth
    pub stack_depth: u32,
}

impl ExecutionPoint {
    /// Create a new execution point
    pub fn new(file: &str, line: u32, column: u32, function: &str, stack_depth: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            function: function.to_string(),
            stack_depth,
        }
    }
}

/// Call stack frame
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// The function name
    pub function: String,
    /// The file location
    pub file: String,
    /// The line number
    pub line: u32,
    /// Local variables in this frame
    pub variables: Vec<String>,
}

impl StackFrame {
    /// Create a new stack frame
    pub fn new(function: &str, file: &str, line: u32) -> Self {
        Self {
            function: function.to_string(),
            file: file.to_string(),
            line,
            variables: Vec::new(),
        }
    }

    /// Add a variable to this stack frame
    pub fn add_variable(&mut self, variable: &str) {
        self.variables.push(variable.to_string());
    }
}

/// Represents the call stack
pub struct CallStack {
    /// The frames in the call stack
    frames: Vec<StackFrame>,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStack {
    /// Create a new empty call stack
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Push a new frame onto the stack
    pub fn push_frame(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    /// Pop the top frame from the stack
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }

    /// Get the current frame
    pub fn current_frame(&self) -> Option<&StackFrame> {
        self.frames.last()
    }

    /// Get a mutable reference to the current frame
    pub fn current_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }

    /// Get all frames in the stack
    pub fn get_frames(&self) -> &[StackFrame] {
        &self.frames
    }

    /// Get the stack depth
    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

/// Manages the flow control of program execution
pub struct FlowController {
    /// The current execution state
    state: ExecutionState,
    /// The next flow control action to take
    next_action: FlowControl,
    /// The current execution point
    current_point: Option<ExecutionPoint>,
    /// The call stack
    call_stack: CallStack,
}

impl Default for FlowController {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowController {
    /// Create a new FlowController
    pub fn new() -> Self {
        Self {
            state: ExecutionState::NotRunning,
            next_action: FlowControl::Stop,
            current_point: None,
            call_stack: CallStack::new(),
        }
    }

    /// Start program execution
    pub fn start(&mut self) {
        self.state = ExecutionState::Running;
        self.next_action = FlowControl::Continue;
        self.call_stack.clear();
    }

    /// Pause program execution
    pub fn pause(&mut self) {
        self.state = ExecutionState::Paused;
    }

    /// Resume execution with the given flow control
    pub fn resume(&mut self, control: FlowControl) {
        self.state = ExecutionState::Running;
        self.next_action = control;
    }

    /// Stop program execution
    pub fn stop(&mut self) {
        self.state = ExecutionState::NotRunning;
        self.next_action = FlowControl::Stop;
        self.call_stack.clear();
        self.current_point = None;
    }

    /// Complete program execution normally
    pub fn complete(&mut self) {
        self.state = ExecutionState::Completed;
        self.next_action = FlowControl::Stop;
    }

    /// Terminate program execution with an error
    pub fn error(&mut self) {
        self.state = ExecutionState::Error;
        self.next_action = FlowControl::Stop;
    }

    /// Update the current execution point
    pub fn update_execution_point(&mut self, point: ExecutionPoint) {
        self.current_point = Some(point);

        // If we're in a stepping mode and we've reached a new line, pause
        if self.next_action == FlowControl::StepOver || self.next_action == FlowControl::StepInto {
            self.state = ExecutionState::Paused;
        }
    }

    /// Enter a function, pushing a new frame onto the call stack
    pub fn enter_function(&mut self, function: &str, file: &str, line: u32) {
        // Create a new stack frame
        let frame = StackFrame::new(function, file, line);

        // Push it onto the call stack
        self.call_stack.push_frame(frame);

        // Update the current execution point
        if let Some(current_point) = &mut self.current_point {
            current_point.function = function.to_string();
            current_point.file = file.to_string();
            current_point.line = line;
            current_point.stack_depth = self.call_stack.depth() as u32;
        } else {
            self.current_point = Some(ExecutionPoint::new(
                file,
                line,
                0,
                function,
                self.call_stack.depth() as u32,
            ));
        }

        // If we're stepping into, pause when entering a function
        if self.next_action == FlowControl::StepInto {
            self.state = ExecutionState::Paused;
        }
    }

    /// Exit a function, popping the top frame from the call stack
    pub fn exit_function(&mut self) -> Option<StackFrame> {
        // Pop the top frame from the call stack
        let frame = self.call_stack.pop_frame();

        // Update the current execution point to the previous frame
        if let Some(parent_frame) = self.call_stack.current_frame() {
            self.current_point = Some(ExecutionPoint::new(
                &parent_frame.file,
                parent_frame.line,
                0,
                &parent_frame.function,
                self.call_stack.depth() as u32,
            ));
        }

        // If we're stepping out, pause when exiting a function
        if self.next_action == FlowControl::StepOut {
            self.state = ExecutionState::Paused;
            self.next_action = FlowControl::Continue;
        }

        frame
    }

    /// Add a variable to the current stack frame
    pub fn add_variable_to_current_frame(&mut self, variable: &str) {
        if let Some(frame) = self.call_stack.current_frame_mut() {
            frame.add_variable(variable);
        }
    }

    /// Get the current execution state
    pub fn get_state(&self) -> ExecutionState {
        self.state
    }

    /// Get the next flow control action
    pub fn get_next_action(&self) -> FlowControl {
        self.next_action
    }

    /// Get the current execution point
    pub fn get_current_point(&self) -> Option<&ExecutionPoint> {
        self.current_point.as_ref()
    }

    /// Get the call stack
    pub fn get_call_stack(&self) -> &CallStack {
        &self.call_stack
    }
}

// Definition for DebugPosition
#[derive(Debug, Clone)]
pub struct DebugPosition {
    /// The file where the breakpoint was hit
    pub file: String,
    /// The line where the breakpoint was hit
    pub line: u32,
    /// The column where the breakpoint was hit
    pub column: u32,
    /// The function where the breakpoint was hit
    pub function: String,
    /// The stack frame index
    pub stack_frame: u32,
    /// Whether this is an async breakpoint
    pub is_async: bool,
    /// The async task ID if this is an async breakpoint
    pub async_task_id: Option<u64>,
}

// Definition for DebuggerState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerState {
    /// The debugger is running
    Running,
    /// The debugger is paused
    Paused,
    /// The debugger is waiting for input
    Waiting,
    /// The debugger has stopped
    Stopped,
}

// Global state for debugger
static DEBUGGER_STATE: Lazy<RwLock<DebuggerState>> =
    Lazy::new(|| RwLock::new(DebuggerState::Stopped));

// Current debug position
static CURRENT_DEBUG_POSITION: Lazy<RwLock<Option<DebugPosition>>> =
    Lazy::new(|| RwLock::new(None));

// Breakpoint manager singleton
pub struct BreakpointManager {
    breakpoints: Vec<Breakpoint>,
    next_id: AtomicU32,
}

impl BreakpointManager {
    /// Create a new breakpoint manager
    fn new() -> Self {
        Self {
            breakpoints: Vec::new(),
            next_id: AtomicU32::new(1),
        }
    }

    /// Get a breakpoint by its location
    pub fn get_breakpoint_by_location(&self, location: &str) -> Option<&Breakpoint> {
        self.breakpoints
            .iter()
            .find(|bp| format!("{}:{}:{}", bp.file, bp.line, bp.column) == location)
    }

    /// Get a mutable reference to a breakpoint by its location
    pub fn get_breakpoint_by_location_mut(&mut self, location: &str) -> Option<&mut Breakpoint> {
        self.breakpoints
            .iter_mut()
            .find(|bp| format!("{}:{}:{}", bp.file, bp.line, bp.column) == location)
    }
}

// Global breakpoint manager
static BREAKPOINT_MANAGER: Lazy<RwLock<BreakpointManager>> =
    Lazy::new(|| RwLock::new(BreakpointManager::new()));

/// Process an async debug point (breakpoint in async code)
pub fn handle_async_breakpoint(
    file: &str,
    line: u32,
    column: u32,
    task_id: u64,
    function_name: &str,
) -> DbugResult<bool> {
    let location = format!("{}:{}:{}", file, line, column);

    // Check if there's a breakpoint at this location
    let should_break = {
        let manager = BREAKPOINT_MANAGER.read().unwrap();
        if let Some(bp) = manager.get_breakpoint_by_location(&location) {
            if bp.enabled {
                // Update hit count
                {
                    let mut mgr = BREAKPOINT_MANAGER.write().unwrap();
                    if let Some(bp) = mgr.get_breakpoint_by_location_mut(&location) {
                        bp.hit_count += 1;
                    }
                }

                // Log the breakpoint hit
                eprintln!(
                    "[DBUG] Async breakpoint hit: {} in {} (task_id: {})",
                    location, function_name, task_id
                );

                // Check if we should pause at this breakpoint (based on conditions, hit counts, etc.)
                process_breakpoint_hit(&location, task_id)
            } else {
                // Breakpoint is disabled
                eprintln!("[DBUG] Async breakpoint is disabled: {}", location);
                false
            }
        } else {
            // No breakpoint at this location
            eprintln!("[DBUG] No async breakpoint defined at {}", location);
            false
        }
    };

    // Update debugger state if we're pausing
    if should_break {
        eprintln!(
            "[DBUG] Pausing at async breakpoint: {} in {} (task_id: {})",
            location, function_name, task_id
        );

        // Update the current debug position
        *CURRENT_DEBUG_POSITION.write().unwrap() = Some(DebugPosition {
            file: file.to_string(),
            line,
            column,
            function: function_name.to_string(),
            stack_frame: 0,
            is_async: true,
            async_task_id: Some(task_id),
        });

        // Update the debugger state
        *DEBUGGER_STATE.write().unwrap() = DebuggerState::Paused;
    }

    Ok(should_break)
}

/// Process a breakpoint hit event to determine if execution should pause
fn process_breakpoint_hit(location: &str, task_id: u64) -> bool {
    // Get the breakpoint details
    let manager = BREAKPOINT_MANAGER.read().unwrap();
    let bp = match manager.get_breakpoint_by_location(location) {
        Some(bp) => bp.clone(),
        None => return false, // No breakpoint found
    };

    // Check hit count conditions
    let hit_count_match = match &bp.condition_mode {
        crate::runtime::BreakpointConditionMode::HitCount(condition) => {
            condition.is_met(bp.hit_count)
        }
        crate::runtime::BreakpointConditionMode::Combined { hit_count, .. } => {
            hit_count.is_met(bp.hit_count)
        }
        _ => true, // No hit count condition
    };

    // If hit count doesn't match, don't pause
    if !hit_count_match {
        return false;
    }

    // Check condition expression if one exists
    match &bp.condition_mode {
        crate::runtime::BreakpointConditionMode::ConditionalExpression(condition) => {
            // Evaluate the condition in the current context
            match evaluate_breakpoint_condition(condition, task_id) {
                Ok(result) => {
                    // Convert result to boolean
                    match result.as_bool() {
                        Ok(should_break) => should_break,
                        Err(_) => {
                            eprintln!(
                                "[DBUG] Error: Breakpoint condition did not evaluate to a boolean"
                            );
                            false
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DBUG] Error evaluating breakpoint condition: {}", e);
                    false
                }
            }
        }
        crate::runtime::BreakpointConditionMode::Combined { expression, .. } => {
            // Evaluate the condition in the current context
            match evaluate_breakpoint_condition(expression, task_id) {
                Ok(result) => {
                    // Convert result to boolean
                    match result.as_bool() {
                        Ok(should_break) => should_break,
                        Err(_) => {
                            eprintln!(
                                "[DBUG] Error: Breakpoint condition did not evaluate to a boolean"
                            );
                            false
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[DBUG] Error evaluating breakpoint condition: {}", e);
                    false
                }
            }
        }
        _ => true, // No condition, always break
    }
}

/// Evaluate a breakpoint condition in the current context
fn evaluate_breakpoint_condition(_condition: &str, task_id: u64) -> DbugResult<VariableValue> {
    // This is a simplified version - in a real implementation,
    // you would evaluate the condition in the current context,
    // with access to variables, etc.

    // For now, just create a dummy result based on the task_id
    // In a real implementation, this would use a proper expression evaluator
    Ok(VariableValue::Boolean(task_id % 2 == 0))
}

// Add an extension trait for VariableValue to support as_bool
trait VariableValueExt {
    fn as_bool(&self) -> Result<bool, String>;
}

impl VariableValueExt for VariableValue {
    fn as_bool(&self) -> Result<bool, String> {
        match self {
            VariableValue::Boolean(b) => Ok(*b),
            VariableValue::Integer(i) => Ok(*i != 0),
            VariableValue::Float(f) => Ok(*f != 0.0),
            VariableValue::String(s) => Ok(!s.is_empty()),
            _ => Err("Cannot convert value to boolean".into()),
        }
    }
}
