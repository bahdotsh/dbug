// Dbug - A CLI-based debugger for Rust projects
//
// This crate provides a simple and intuitive debugging experience
// for Rust developers.

// Temporarily commented out due to compilation issues
// pub mod cli;
pub mod cargo;
pub mod communication;
pub mod compiler;
pub mod errors;
pub mod instrumentation;
pub mod prelude;
pub mod runtime;
pub mod session;
pub mod source;
pub mod tui;
pub mod utils;

/// This module contains internal implementation details
/// Not intended for direct use by end users
pub mod _internal {
    use crate::errors::DbugResult;
    use crate::runtime::async_support::{
        generate_async_task_id as runtime_generate_task_id,
        get_current_async_task_id as runtime_get_task_id, TaskId,
    };
    use std::sync::Once;

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
        if let Err(e) =
            crate::communication::process_debug_point(file, line, column, &current_function)
        {
            eprintln!("[DBUG] Error processing debug point: {}", e);
        }

        // Also log to console in development mode
        eprintln!(
            "[DBUG] Breakpoint at {}:{}:{} in {}",
            file, line, column, current_function
        );
    }

    /// Register a variable with the debugger
    pub fn register_variable(
        name: &str,
        type_name: &str,
        value: &str,
        is_mutable: bool,
    ) -> DbugResult<()> {
        crate::communication::notify_variable_changed(name, type_name, value, is_mutable)
    }

    /// Generate a unique ID for an async task
    pub fn generate_async_task_id() -> TaskId {
        runtime_generate_task_id()
    }

    /// Get the ID of the current async task
    pub fn get_current_async_task_id() -> TaskId {
        runtime_get_task_id()
    }

    /// Called when entering an async function that's marked for debugging
    pub fn enter_async_function(function_name: &str, task_id: TaskId) {
        init();

        // Get the current file and line number
        let file = std::panic::Location::caller().file();
        let line = std::panic::Location::caller().line();

        // Notify the debugger
        if let Err(e) = crate::communication::notify_async_function_entered(function_name, task_id)
        {
            eprintln!("[DBUG] Error notifying async function entry: {}", e);
        }

        // Register the async task
        if let Err(e) =
            crate::runtime::async_support::register_async_task(function_name, task_id, None)
        {
            eprintln!("[DBUG] Error registering async task: {}", e);
        }

        // Also log to console in development mode
        eprintln!(
            "[DBUG] Entering async function: {} (task_id: {})",
            function_name, task_id
        );
    }

    /// Called when exiting an async function that's marked for debugging
    pub fn exit_async_function(function_name: &str, task_id: TaskId) {
        // Notify the debugger
        if let Err(e) = crate::communication::notify_async_function_exited(function_name, task_id) {
            eprintln!("[DBUG] Error notifying async function exit: {}", e);
        }

        // Mark the task as completed
        if let Err(e) = crate::runtime::async_support::complete_async_task(task_id) {
            eprintln!("[DBUG] Error completing async task: {}", e);
        }

        // Also log to console in development mode
        eprintln!(
            "[DBUG] Exiting async function: {} (task_id: {})",
            function_name, task_id
        );
    }

    /// Called when an async breakpoint is encountered
    pub fn async_break_point(file: &str, line: u32, column: u32, task_id: TaskId) {
        init();

        // Use a simpler approach - we'll use the file name to guess the function
        // In a real implementation, this would use DWARF debug info to get the actual function name
        let file_stem = std::path::Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let current_function = format!("async_function_in_{}", file_stem);

        // Process the debug point
        if let Err(e) = crate::communication::process_async_debug_point(file, line, column, task_id)
        {
            eprintln!("[DBUG] Error processing async debug point: {}", e);
        }

        // Also log to console in development mode
        eprintln!(
            "[DBUG] Async breakpoint at {}:{}:{} in {} (task_id: {})",
            file, line, column, current_function, task_id
        );
    }
}

// Re-export the prelude for convenience
pub use prelude::*;
