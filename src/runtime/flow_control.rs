// Flow control functionality for the runtime debugger

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
        if self.next_action == FlowControl::StepOver || 
           self.next_action == FlowControl::StepInto {
            self.state = ExecutionState::Paused;
        }
    }

    /// Enter a function, pushing a new frame onto the call stack
    pub fn enter_function(&mut self, function: &str, file: &str, line: u32) {
        let frame = StackFrame::new(function, file, line);
        self.call_stack.push_frame(frame);
        
        // If we're stepping into, pause when entering a function
        if self.next_action == FlowControl::StepInto {
            self.state = ExecutionState::Paused;
        }
    }

    /// Exit a function, popping the top frame from the call stack
    pub fn exit_function(&mut self) {
        self.call_stack.pop_frame();
        
        // If we're stepping out, pause when exiting a function
        if self.next_action == FlowControl::StepOut && !self.call_stack.get_frames().is_empty() {
            self.state = ExecutionState::Paused;
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