// Communication module for the dbug debugger
//
// This module provides communication mechanisms between the debugger and the instrumented code.

use std::env;
use std::fs::{OpenOptions, remove_file};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use crate::errors::{DbugError, DbugResult};
use std::collections::VecDeque;
use memmap2::{MmapMut, MmapOptions};

// Maximum time to wait for a response from the debugger
const RESPONSE_TIMEOUT_MS: u64 = 5000;
// Maximum message batch size before forced flush
const MAX_BATCH_SIZE: usize = 10;
// The size of the memory-mapped file (8KB should be sufficient for most messages)
const MMAP_SIZE: usize = 8192;

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
    /// Multiple messages batched together for efficiency
    BatchedMessages(Vec<DebuggerMessage>),
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

/// Flag to indicate if a batch flush is in progress
static BATCH_FLUSH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Handles communication between the debugger and the instrumented code
pub struct CommunicationChannel {
    /// Path to the message file
    message_file_path: PathBuf,
    /// Path to the response file
    response_file_path: PathBuf,
    /// Whether the channel is active
    active: bool,
    /// Memory-mapped message file for faster writes
    message_mmap: Option<MmapMut>,
    /// Memory-mapped response file for faster reads
    response_mmap: Option<MmapMut>,
    /// Message queue for batching
    message_queue: VecDeque<DebuggerMessage>,
    /// Last flush time
    last_flush: Instant,
}

impl CommunicationChannel {
    /// Create a new communication channel
    pub fn new() -> DbugResult<Self> {
        // Create temporary files for communication
        let temp_dir = env::temp_dir();
        let pid = std::process::id();
        let message_file_path = temp_dir.join(format!("dbug_message_{}.json", pid));
        let response_file_path = temp_dir.join(format!("dbug_response_{}.json", pid));
        
        // Create the files if they don't exist
        let message_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&message_file_path)
            .map_err(|e| DbugError::CommunicationError(format!("Failed to create message file: {}", e)))?;
        
        message_file.set_len(MMAP_SIZE as u64)
            .map_err(|e| DbugError::CommunicationError(format!("Failed to set message file size: {}", e)))?;
            
        let response_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&response_file_path)
            .map_err(|e| DbugError::CommunicationError(format!("Failed to create response file: {}", e)))?;
            
        response_file.set_len(MMAP_SIZE as u64)
            .map_err(|e| DbugError::CommunicationError(format!("Failed to set response file size: {}", e)))?;
        
        // Create memory maps for faster access
        let message_mmap = unsafe { MmapOptions::new().map_mut(&message_file) }
            .map_err(|e| DbugError::CommunicationError(format!("Failed to create memory map for message file: {}", e)))?;
            
        let response_mmap = unsafe { MmapOptions::new().map_mut(&response_file) }
            .map_err(|e| DbugError::CommunicationError(format!("Failed to create memory map for response file: {}", e)))?;
        
        Ok(Self {
            message_file_path,
            response_file_path,
            active: true,
            message_mmap: Some(message_mmap),
            response_mmap: Some(response_mmap),
            message_queue: VecDeque::with_capacity(MAX_BATCH_SIZE),
            last_flush: Instant::now(),
        })
    }
    
    /// Queue a message to be sent to the debugger
    pub fn queue_message(&mut self, message: DebuggerMessage) -> DbugResult<()> {
        if !self.active {
            return Ok(());
        }
        
        // Add the message to the queue
        self.message_queue.push_back(message);
        
        // Flush if we've reached the batch size or if it's a breakpoint (which needs immediate attention)
        let should_flush = self.message_queue.len() >= MAX_BATCH_SIZE ||
                         self.message_queue.iter().any(|m| matches!(m, DebuggerMessage::BreakpointHit { .. })) ||
                         self.last_flush.elapsed() > Duration::from_millis(100);
                         
        if should_flush {
            self.flush_message_queue()?;
        }
        
        Ok(())
    }
    
    /// Flush the message queue
    fn flush_message_queue(&mut self) -> DbugResult<()> {
        if self.message_queue.is_empty() || !self.active {
            return Ok(());
        }
        
        // Set the flag to avoid recursive flushes
        if BATCH_FLUSH_IN_PROGRESS.swap(true, Ordering::SeqCst) {
            // Another flush is in progress, skip this one
            return Ok(());
        }
        
        let messages: Vec<_> = self.message_queue.drain(..).collect();
        
        // If there's only one message, send it directly, otherwise batch them
        let message_to_send = if messages.len() == 1 {
            messages.into_iter().next().unwrap()
        } else {
            DebuggerMessage::BatchedMessages(messages)
        };
        
        let result = self.send_message_internal(&message_to_send);
        
        // Reset the flag
        BATCH_FLUSH_IN_PROGRESS.store(false, Ordering::SeqCst);
        
        // Update the last flush time
        self.last_flush = Instant::now();
        
        result
    }
    
    /// Internal method to send a message to the debugger
    fn send_message_internal(&mut self, message: &DebuggerMessage) -> DbugResult<()> {
        if !self.active {
            return Ok(());
        }
        
        let json = serde_json::to_string(message)
            .map_err(DbugError::JsonParse)?;
            
        if json.len() + 1 > MMAP_SIZE {  // +1 for null terminator
            return Err(DbugError::CommunicationError(
                format!("Message too large for buffer: {} bytes", json.len())
            ));
        }
        
        if let Some(mmap) = self.message_mmap.as_mut() {
            // Clear the memory map first
            mmap.fill(0);
            
            // Write the message
            mmap[..json.len()].copy_from_slice(json.as_bytes());
            
            // Flush the memory map to ensure it's written to disk
            mmap.flush()
                .map_err(|e| DbugError::CommunicationError(format!("Failed to flush memory map: {}", e)))?;
        } else {
            return Err(DbugError::CommunicationError("Memory map not initialized".into()));
        }
        
        Ok(())
    }
    
    /// Wait for a response from the debugger
    pub fn wait_for_response(&mut self) -> DbugResult<Option<DebuggerResponse>> {
        if !self.active {
            return Ok(None);
        }
        
        // Make sure all pending messages are sent
        self.flush_message_queue()?;
        
        let start_time = Instant::now();
        
        while start_time.elapsed() < Duration::from_millis(RESPONSE_TIMEOUT_MS) {
            if let Some(mmap) = self.response_mmap.as_ref() {
                // Find the null terminator that marks the end of the JSON string
                let mut content_length = 0;
                for i in 0..MMAP_SIZE {
                    if mmap[i] == 0 {
                        content_length = i;
                        break;
                    }
                }
                
                if content_length == 0 {
                    // No response yet, wait a bit and try again
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                
                // Try to parse the response
                match std::str::from_utf8(&mmap[..content_length]) {
                    Ok(content) if !content.is_empty() => {
                        match serde_json::from_str::<DebuggerResponse>(content) {
                            Ok(response) => {
                                // Clear the response area
                                if let Some(mmap_mut) = self.response_mmap.as_mut() {
                                    mmap_mut.fill(0);
                                    mmap_mut.flush().map_err(|e| 
                                        DbugError::CommunicationError(format!("Failed to flush response memory map: {}", e))
                                    )?;
                                }
                                
                                return Ok(Some(response));
                            }
                            Err(_) => {
                                // Invalid response, wait a bit and try again
                                std::thread::sleep(Duration::from_millis(10));
                                continue;
                            }
                        }
                    }
                    _ => {
                        // No response or invalid UTF-8, wait a bit and try again
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                }
            } else {
                return Err(DbugError::CommunicationError("Response memory map not initialized".into()));
            }
        }
        
        // Timeout reached
        Err(DbugError::ResponseTimeout)
    }
    
    /// Close the communication channel
    pub fn close(&mut self) -> DbugResult<()> {
        // Flush any remaining messages
        let _ = self.flush_message_queue();
        
        self.active = false;
        
        // Drop the memory maps
        self.message_mmap = None;
        self.response_mmap = None;
        
        // Remove the files
        if self.message_file_path.exists() {
            remove_file(&self.message_file_path)
                .map_err(|e| DbugError::CommunicationError(format!("Failed to remove message file: {}", e)))?;
        }
        
        if self.response_file_path.exists() {
            remove_file(&self.response_file_path)
                .map_err(|e| DbugError::CommunicationError(format!("Failed to remove response file: {}", e)))?;
        }
        
        Ok(())
    }
}

impl Drop for CommunicationChannel {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Queue a message to be sent to the debugger
pub fn queue_message(message: DebuggerMessage) -> DbugResult<()> {
    if let Ok(mut channel) = COMMUNICATION_CHANNEL.lock() {
        channel.queue_message(message)
    } else {
        Err(DbugError::CommunicationError("Failed to acquire lock on communication channel".into()))
    }
}

/// Send a message to the debugger immediately
pub fn send_message(message: DebuggerMessage) -> DbugResult<()> {
    if let Ok(mut channel) = COMMUNICATION_CHANNEL.lock() {
        // Queue and force an immediate flush
        channel.queue_message(message)?;
        channel.flush_message_queue()
    } else {
        Err(DbugError::CommunicationError("Failed to acquire lock on communication channel".into()))
    }
}

/// Wait for a response from the debugger
pub fn wait_for_response() -> DbugResult<Option<DebuggerResponse>> {
    if let Ok(mut channel) = COMMUNICATION_CHANNEL.lock() {
        channel.wait_for_response()
    } else {
        Err(DbugError::CommunicationError("Failed to acquire lock on communication channel".into()))
    }
}

/// Process a debug point in the code
pub fn process_debug_point(file: &str, line: u32, column: u32, function: &str) -> DbugResult<()> {
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
pub fn notify_function_entered(function: &str, file: &str, line: u32) -> DbugResult<()> {
    let message = DebuggerMessage::FunctionEntered {
        function: function.to_string(),
        file: file.to_string(),
        line,
    };
    
    queue_message(message)
}

/// Notify the debugger that a function has been exited
pub fn notify_function_exited(function: &str) -> DbugResult<()> {
    let message = DebuggerMessage::FunctionExited {
        function: function.to_string(),
    };
    
    queue_message(message)
}

/// Notify the debugger that a variable has been changed
pub fn notify_variable_changed(name: &str, type_name: &str, value: &str, is_mutable: bool) -> DbugResult<()> {
    let message = DebuggerMessage::VariableChanged {
        name: name.to_string(),
        type_name: type_name.to_string(),
        value: value.to_string(),
        is_mutable,
    };
    
    queue_message(message)
} 