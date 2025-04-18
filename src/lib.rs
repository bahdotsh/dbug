// Dbug - A CLI-based debugger for Rust projects
//
// This crate provides a simple and intuitive debugging experience
// for Rust developers.

pub mod cli;
pub mod compiler;
pub mod runtime;
pub mod instrumentation;
pub mod utils;

/// This module contains internal implementation details
/// Not intended for direct use by end users
pub mod _internal {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    /// Initialize the debugging runtime
    pub fn init() {
        INIT.call_once(|| {
            // Initialize the debugging runtime
            // This will be expanded in the future
        });
    }
    
    /// Called when entering a function that's marked for debugging
    pub fn enter_function(function_name: &str) {
        init();
        // For now, just print a message
        eprintln!("[DBUG] Entering function: {}", function_name);
    }
    
    /// Called when exiting a function that's marked for debugging
    pub fn exit_function(function_name: &str) {
        // For now, just print a message
        eprintln!("[DBUG] Exiting function: {}", function_name);
    }
    
    /// Called when a breakpoint is encountered
    pub fn break_point(file: &str, line: u32, column: u32) {
        init();
        eprintln!("[DBUG] Breakpoint at {}:{}:{}", file, line, column);
        // This will be expanded to actually pause execution and provide a debugging interface
    }
}

/// A collection of commonly used items
pub mod prelude {
    // Re-export the macros from the proc-macro crate
    pub use dbug_macros::*;
    
    // Re-export other commonly used items
    // This will be expanded in the future
} 