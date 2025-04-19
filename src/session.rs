//! Debugger session management module
//!
//! This module provides functionality for managing debugger sessions,
//! including tracking state across multiple runs and handling breakpoints.

use std::sync::{Mutex, Arc};
use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;
use crate::errors::{DbugResult, DbugError};

/// The current debugging session
static CURRENT_SESSION: Lazy<Arc<Mutex<DebugSession>>> = Lazy::new(|| {
    Arc::new(Mutex::new(DebugSession::new()))
});

/// Represents a debugging session
pub struct DebugSession {
    /// The project being debugged
    project_path: Option<PathBuf>,
    /// Whether the session is active
    active: bool,
    /// The executable being debugged
    executable_path: Option<PathBuf>,
    /// Process ID of the debugged program
    debugged_pid: Option<u32>,
    /// Breakpoints that have been set
    breakpoints: Vec<(String, u32)>, // (file, line)
}

impl DebugSession {
    /// Create a new debugging session
    pub fn new() -> Self {
        Self {
            project_path: None,
            active: false,
            executable_path: None,
            debugged_pid: None,
            breakpoints: Vec::new(),
        }
    }
    
    /// Start a debugging session for the given project
    pub fn start(&mut self, project_path: &str) -> DbugResult<()> {
        if self.active {
            return Err(DbugError::CliError("A debugging session is already active".to_string()));
        }
        
        self.project_path = Some(PathBuf::from(project_path));
        self.active = true;
        self.breakpoints.clear();
        
        Ok(())
    }
    
    /// Stop the current debugging session
    pub fn stop(&mut self) -> DbugResult<()> {
        if !self.active {
            return Err(DbugError::CliError("No active debugging session".to_string()));
        }
        
        // Terminate the debugged process if it's still running
        if let Some(pid) = self.debugged_pid {
            // On Unix-like systems
            #[cfg(unix)]
            {
                use std::process::Command;
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(&pid.to_string())
                    .status();
            }
            
            // On Windows
            #[cfg(windows)]
            {
                use std::process::Command;
                let _ = Command::new("taskkill")
                    .args(&["/PID", &pid.to_string(), "/F"])
                    .status();
            }
        }
        
        self.active = false;
        self.debugged_pid = None;
        
        Ok(())
    }
    
    /// Set the executable path for the session
    pub fn set_executable(&mut self, path: &Path) -> DbugResult<()> {
        if !self.active {
            return Err(DbugError::CliError("No active debugging session".to_string()));
        }
        
        self.executable_path = Some(path.to_path_buf());
        Ok(())
    }
    
    /// Set the process ID of the debugged program
    pub fn set_debugged_pid(&mut self, pid: u32) -> DbugResult<()> {
        if !self.active {
            return Err(DbugError::CliError("No active debugging session".to_string()));
        }
        
        self.debugged_pid = Some(pid);
        Ok(())
    }
    
    /// Add a breakpoint at the specified file and line
    pub fn add_breakpoint(&mut self, file: &str, line: u32) -> DbugResult<()> {
        if !self.active {
            return Err(DbugError::CliError("No active debugging session".to_string()));
        }
        
        self.breakpoints.push((file.to_string(), line));
        Ok(())
    }
    
    /// Remove a breakpoint at the specified file and line
    pub fn remove_breakpoint(&mut self, file: &str, line: u32) -> DbugResult<()> {
        if !self.active {
            return Err(DbugError::CliError("No active debugging session".to_string()));
        }
        
        let pos = self.breakpoints.iter().position(|(f, l)| f == file && *l == line);
        if let Some(pos) = pos {
            self.breakpoints.remove(pos);
            Ok(())
        } else {
            Err(DbugError::CliError(format!("No breakpoint at {}:{}", file, line)))
        }
    }
    
    /// Get the current project path
    pub fn get_project_path(&self) -> Option<&Path> {
        self.project_path.as_deref()
    }
    
    /// Get the executable path
    pub fn get_executable_path(&self) -> Option<&Path> {
        self.executable_path.as_deref()
    }
    
    /// Get the process ID of the debugged program
    pub fn get_debugged_pid(&self) -> Option<u32> {
        self.debugged_pid
    }
    
    /// Get the list of breakpoints
    pub fn get_breakpoints(&self) -> &[(String, u32)] {
        &self.breakpoints
    }
    
    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// Get the current debugging session
pub fn get_current_session() -> DbugResult<Arc<Mutex<DebugSession>>> {
    Ok(CURRENT_SESSION.clone())
} 