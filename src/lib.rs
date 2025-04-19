// Dbug - A CLI-based debugger for Rust projects
//
// This crate provides a simple and intuitive debugging experience
// for Rust developers.

// Temporarily commented out due to compilation issues
// pub mod cli;
pub mod compiler;
pub mod runtime;
pub mod instrumentation;
pub mod utils;
pub mod communication;
pub mod errors;
pub mod session;
pub mod cargo;
pub mod source;

/// This module contains internal implementation details
/// Not intended for direct use by end users
pub mod _internal {
    use std::sync::Once;
    use crate::errors::DbugResult;
    
    static INIT: Once = Once::new();
    
    /// Initialize the debugging runtime
    pub fn init() {
        INIT.call_once(|| {
            // Initialize the debugging runtime
            // Any one-time initialization goes here
            eprintln!("[DBUG] Initializing debug runtime");
        });
    }
    
    /// Called when entering a function that's marked for debugging
    pub fn enter_function(function_name: &str) {
        init();
        
        // Get the current file and line number
        let file = std::panic::Location::caller().file();
        let line = std::panic::Location::caller().line();
        
        // Notify the debugger
        if let Err(e) = crate::communication::notify_function_entered(function_name, file, line) {
            eprintln!("[DBUG] Error notifying function entry: {}", e);
        }
        
        // Also log to console in development mode
        eprintln!("[DBUG] Entering function: {}", function_name);
    }
    
    /// Called when exiting a function that's marked for debugging
    pub fn exit_function(function_name: &str) {
        // Notify the debugger
        if let Err(e) = crate::communication::notify_function_exited(function_name) {
            eprintln!("[DBUG] Error notifying function exit: {}", e);
        }
        
        // Also log to console in development mode
        eprintln!("[DBUG] Exiting function: {}", function_name);
    }
    
    /// Called when a breakpoint is encountered
    pub fn break_point(file: &str, line: u32, column: u32) {
        init();
        
        // Use a simpler approach - we'll use the file name to guess the function
        // In a real implementation, this would use DWARF debug info to get the actual function name
        let file_stem = std::path::Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let current_function = format!("function_in_{}", file_stem);
        
        // Process the debug point
        if let Err(e) = crate::communication::process_debug_point(file, line, column, &current_function) {
            eprintln!("[DBUG] Error processing debug point: {}", e);
        }
        
        // Also log to console in development mode
        eprintln!("[DBUG] Breakpoint at {}:{}:{} in {}", file, line, column, current_function);
    }
    
    /// Register a variable with the debugger
    pub fn register_variable(name: &str, type_name: &str, value: &str, is_mutable: bool) -> DbugResult<()> {
        crate::communication::notify_variable_changed(name, type_name, value, is_mutable)
    }
}

/// A collection of commonly used items
pub mod prelude {
    // Re-export the macros from the proc-macro crate
    
    
    // Re-export specific macros by their correct names
    pub use dbug_macros::dbug;
    pub use dbug_macros::break_here;
    pub use dbug_macros::break_at;
    pub use dbug_macros::register_var;
    
    // Re-export runtime types that might be useful in user code
    pub use crate::runtime::{Variable, VariableValue};
    
    // Re-export the register_variable function
    pub use crate::_internal::register_variable;
    
    // Re-export error types and utilities
    pub use crate::errors::{DbugError, DbugResult, ErrorExt};
} 