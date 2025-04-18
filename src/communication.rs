// Communication module for the dbug debugger
//
// This module provides communication mechanisms between the debugger and the instrumented code.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

// Maximum time to wait for a response from the debugger
const RESPONSE_TIMEOUT_MS: u64 = 5000;

/// A message from the instrumented code to the debugger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebuggerMessage {
    /// A breakpoint has been hit
    BreakpointHit {
        /// The file where the breakpoint was hit
        file: String,
        /// The line where the breakpoint was hit
        line: u32,
        /// The column where the breakpoint was hit
        column: u32,
        /// The function where the breakpoint was hit
        function: String,
    },
    /// A function has been entered
    FunctionEntered {
        /// The name of the function
        function: String,
        /// The file containing the function
        file: String,
        /// The line where the function starts
        line: u32,
    },
    /// A function has been exited
    FunctionExited {
        /// The name of the function
        function: String,
    },
    /// A variable has been created or modified
    VariableChanged {
        /// The name of the variable
        name: String,
        /// The type of the variable
        type_name: String,
        /// The value of the variable as a string
        value: String,
        /// Whether the variable is mutable
        is_mutable: bool,
    },
}

/// A response from the debugger to the instrumented code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebuggerResponse {
    /// Continue execution
    Continue,
    /// Step over to the next line
    StepOver,
    /// Step into a function
    StepInto,
    /// Step out of the current function
    StepOut,
    /// Evaluate an expression and return the result
    Evaluate {
        /// The expression to evaluate
        expression: String,
    },
}

/// Global communication channel instance
static COMMUNICATION_CHANNEL: Lazy<Arc<Mutex<CommunicationChannel>>> = Lazy::new(|| {
    Arc::new(Mutex::new(CommunicationChannel::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialize communication channel: {}", e);
        std::process::exit(1);
    })))
});

/// Handles communication between the debugger and the instrumented code
pub struct CommunicationChannel {
    /// Path to the message file
    message_file_path: PathBuf,
    /// Path to the response file
    response_file_path: PathBuf,
    /// Whether the channel is active
    active: bool,
}

impl CommunicationChannel {
    /// Create a new communication channel
    pub fn new() -> io::Result<Self> {
        // Create temporary files for communication
        let temp_dir = env::temp_dir();
        let pid = std::process::id();
        let message_file_path = temp_dir.join(format!("dbug_message_{}.json", pid));
        let response_file_path = temp_dir.join(format!("dbug_response_{}.json", pid));
        
        // Create the files if they don't exist
        File::create(&message_file_path)?;
        File::create(&response_file_path)?;
        
        Ok(Self {
            message_file_path,
            response_file_path,
            active: true,
        })
    }
    
    /// Send a message to the debugger
    pub fn send_message(&self, message: &DebuggerMessage) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        
        let json = serde_json::to_string(message)?;
        let mut file = OpenOptions::new().write(true).truncate(true).open(&self.message_file_path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
    
    /// Wait for a response from the debugger
    pub fn wait_for_response(&self) -> io::Result<Option<DebuggerResponse>> {
        if !self.active {
            return Ok(None);
        }
        
        let start_time = Instant::now();
        
        while start_time.elapsed() < Duration::from_millis(RESPONSE_TIMEOUT_MS) {
            let mut file = match File::open(&self.response_file_path) {
                Ok(f) => f,
                Err(_) => {
                    // If the file doesn't exist yet, wait a bit and try again
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            };
            
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            
            if content.is_empty() {
                // No response yet, wait a bit and try again
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            
            // Try to parse the response
            match serde_json::from_str::<DebuggerResponse>(&content) {
                Ok(response) => {
                    // Clear the response file
                    let mut file = OpenOptions::new().write(true).truncate(true).open(&self.response_file_path)?;
                    file.seek(SeekFrom::Start(0))?;
                    file.set_len(0)?;
                    
                    return Ok(Some(response));
                }
                Err(_) => {
                    // Invalid response, wait a bit and try again
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
            }
        }
        
        // Timeout reached
        Ok(None)
    }
    
    /// Close the communication channel
    pub fn close(&mut self) -> io::Result<()> {
        self.active = false;
        
        // Remove the files
        if self.message_file_path.exists() {
            fs::remove_file(&self.message_file_path)?;
        }
        
        if self.response_file_path.exists() {
            fs::remove_file(&self.response_file_path)?;
        }
        
        Ok(())
    }
}

impl Drop for CommunicationChannel {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Send a message to the debugger
pub fn send_message(message: DebuggerMessage) -> io::Result<()> {
    if let Ok(channel) = COMMUNICATION_CHANNEL.lock() {
        channel.send_message(&message)
    } else {
        Ok(())
    }
}

/// Wait for a response from the debugger
pub fn wait_for_response() -> io::Result<Option<DebuggerResponse>> {
    if let Ok(channel) = COMMUNICATION_CHANNEL.lock() {
        channel.wait_for_response()
    } else {
        Ok(None)
    }
}

/// Process a debug point in the code
pub fn process_debug_point(file: &str, line: u32, column: u32, function: &str) -> io::Result<()> {
    // Send a breakpoint hit message
    let message = DebuggerMessage::BreakpointHit {
        file: file.to_string(),
        line,
        column,
        function: function.to_string(),
    };
    
    send_message(message)?;
    
    // Wait for a response
    match wait_for_response()? {
        Some(DebuggerResponse::Continue) => {
            // Just continue
        }
        Some(DebuggerResponse::StepOver) => {
            // Continue with step over
        }
        Some(DebuggerResponse::StepInto) => {
            // Continue with step into
        }
        Some(DebuggerResponse::StepOut) => {
            // Continue with step out
        }
        Some(DebuggerResponse::Evaluate { expression }) => {
            // TODO: Evaluate the expression
            // For now, just ignore it
            println!("Evaluation not implemented: {}", expression);
        }
        None => {
            // No response, just continue
        }
    }
    
    Ok(())
}

/// Notify the debugger that a function has been entered
pub fn notify_function_entered(function: &str, file: &str, line: u32) -> io::Result<()> {
    let message = DebuggerMessage::FunctionEntered {
        function: function.to_string(),
        file: file.to_string(),
        line,
    };
    
    send_message(message)
}

/// Notify the debugger that a function has been exited
pub fn notify_function_exited(function: &str) -> io::Result<()> {
    let message = DebuggerMessage::FunctionExited {
        function: function.to_string(),
    };
    
    send_message(message)
}

/// Notify the debugger that a variable has changed
pub fn notify_variable_changed(name: &str, type_name: &str, value: &str, is_mutable: bool) -> io::Result<()> {
    let message = DebuggerMessage::VariableChanged {
        name: name.to_string(),
        type_name: type_name.to_string(),
        value: value.to_string(),
        is_mutable,
    };
    
    send_message(message)
} 