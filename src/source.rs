// Source file handling module for the dbug debugger

use std::fs;
use std::path::Path;
use crate::errors::{DbugResult, DbugError};

/// Represents a source code file that can be loaded and parsed
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// The path to the source file
    pub path: String,
    /// The content of the source file
    pub content: String,
    /// The lines of the source file
    pub lines: Vec<String>,
}

impl SourceFile {
    /// Load a source file from disk
    pub fn load(path: String) -> DbugResult<Self> {
        let content = fs::read_to_string(&path)
            .map_err(|e| DbugError::Io(e))?;
        
        let lines = content.lines()
            .map(|line| line.to_string())
            .collect();
        
        Ok(Self {
            path,
            content,
            lines,
        })
    }
    
    /// Get a specific line from the source file
    pub fn get_line(&self, line_number: usize) -> Option<&String> {
        if line_number == 0 {
            return None; // Line numbers are 1-indexed
        }
        
        self.lines.get(line_number - 1)
    }
    
    /// Save the source file to disk
    pub fn save(&self, path: &Path) -> DbugResult<()> {
        fs::write(path, &self.content)
            .map_err(|e| DbugError::Io(e))?;
        
        Ok(())
    }
    
    /// Get a range of lines from the source file
    pub fn get_line_range(&self, start: usize, end: usize) -> Vec<(usize, &String)> {
        let mut result = Vec::new();
        
        let start_idx = if start == 0 { 0 } else { start - 1 };
        let end_idx = if end == 0 { 0 } else { end.min(self.lines.len()) };
        
        for i in start_idx..end_idx {
            result.push((i + 1, &self.lines[i]));
        }
        
        result
    }
} 