// Source mapping module for tracking the relationship between
// original source code and instrumented code.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::errors::{DbugResult, DbugError};

/// Global source map instance
static SOURCE_MAP: Lazy<Arc<Mutex<SourceMap>>> = Lazy::new(|| {
    Arc::new(Mutex::new(SourceMap::new()))
});

/// Represents a mapping between original and instrumented source locations
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Original source file
    pub original_file: PathBuf,
    /// Original line number
    pub original_line: u32,
    /// Original column number
    pub original_column: u32,
    /// Instrumented file (may be the same as original)
    pub instrumented_file: PathBuf,
    /// Instrumented line number
    pub instrumented_line: u32,
    /// Instrumented column number
    pub instrumented_column: u32,
}

impl SourceLocation {
    /// Create a new source location mapping
    pub fn new(
        original_file: &Path,
        original_line: u32,
        original_column: u32,
        instrumented_file: &Path,
        instrumented_line: u32,
        instrumented_column: u32,
    ) -> Self {
        Self {
            original_file: original_file.to_path_buf(),
            original_line,
            original_column,
            instrumented_file: instrumented_file.to_path_buf(),
            instrumented_line,
            instrumented_column,
        }
    }
}

/// Source code context around a specific line
#[derive(Debug, Clone)]
pub struct SourceContext {
    /// File containing the source code
    pub file: PathBuf,
    /// The target line number
    pub line: u32,
    /// Lines of source code (line number -> content)
    pub lines: HashMap<u32, String>,
    /// Start line number in the context
    pub start_line: u32,
    /// End line number in the context
    pub end_line: u32,
}

impl SourceContext {
    /// Create an empty source context
    pub fn empty(file: &Path, line: u32) -> Self {
        Self {
            file: file.to_path_buf(),
            line,
            lines: HashMap::new(),
            start_line: line,
            end_line: line,
        }
    }
    
    /// Load a source context from a file with N lines of context
    pub fn load(file: &Path, line: u32, context_lines: u32) -> DbugResult<Self> {
        let content = fs::read_to_string(file)
            .map_err(|e| DbugError::Io(e))?;
        
        let mut lines = HashMap::new();
        let line_count = content.lines().count() as u32;
        
        // Calculate the range of lines to include in the context
        let start_line = line.saturating_sub(context_lines);
        let end_line = std::cmp::min(line + context_lines, line_count);
        
        // Load the lines into the map
        for (i, line_content) in content.lines().enumerate() {
            let line_num = i as u32 + 1; // 1-indexed line numbers
            if line_num >= start_line && line_num <= end_line {
                lines.insert(line_num, line_content.to_string());
            }
        }
        
        Ok(Self {
            file: file.to_path_buf(),
            line,
            lines,
            start_line,
            end_line,
        })
    }
    
    /// Get all the context lines as a vector of (line_number, content) pairs
    pub fn get_lines(&self) -> Vec<(u32, &String)> {
        let mut lines: Vec<_> = self.lines.iter()
            .map(|(line_num, content)| (*line_num, content))
            .collect();
        
        // Sort by line number
        lines.sort_by_key(|(line_num, _)| *line_num);
        lines
    }
    
    /// Get a specific line from the context
    pub fn get_line(&self, line_num: u32) -> Option<&String> {
        self.lines.get(&line_num)
    }
    
    /// Check if a line number is in this context
    pub fn contains_line(&self, line_num: u32) -> bool {
        self.lines.contains_key(&line_num)
    }
}

/// Maps between original source code and instrumented code
pub struct SourceMap {
    /// Maps from original location to instrumented location
    /// Key format: (file, line, column)
    original_to_instrumented: HashMap<(String, u32, u32), SourceLocation>,
    
    /// Maps from instrumented location to original location
    /// Key format: (file, line, column)
    instrumented_to_original: HashMap<(String, u32, u32), SourceLocation>,
    
    /// Cache of loaded source files
    source_cache: HashMap<PathBuf, Vec<String>>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self {
            original_to_instrumented: HashMap::new(),
            instrumented_to_original: HashMap::new(),
            source_cache: HashMap::new(),
        }
    }
    
    /// Add a mapping between original and instrumented source locations
    pub fn add_mapping(&mut self, location: SourceLocation) {
        let original_key = (
            location.original_file.to_string_lossy().to_string(),
            location.original_line,
            location.original_column,
        );
        
        let instrumented_key = (
            location.instrumented_file.to_string_lossy().to_string(),
            location.instrumented_line,
            location.instrumented_column,
        );
        
        self.original_to_instrumented.insert(original_key, location.clone());
        self.instrumented_to_original.insert(instrumented_key, location);
    }
    
    /// Find the instrumented location for an original source location
    pub fn find_instrumented_location(&self, file: &str, line: u32, column: u32) -> Option<&SourceLocation> {
        let key = (file.to_string(), line, column);
        self.original_to_instrumented.get(&key)
    }
    
    /// Find the original location for an instrumented source location
    pub fn find_original_location(&self, file: &str, line: u32, column: u32) -> Option<&SourceLocation> {
        let key = (file.to_string(), line, column);
        self.instrumented_to_original.get(&key)
    }
    
    /// Load source code into the cache
    pub fn load_source_file(&mut self, file_path: &Path) -> DbugResult<()> {
        if self.source_cache.contains_key(file_path) {
            return Ok(());
        }
        
        let content = fs::read_to_string(file_path)
            .map_err(|e| DbugError::Io(e))?;
        
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        self.source_cache.insert(file_path.to_path_buf(), lines);
        
        Ok(())
    }
    
    /// Get a source context from a file
    pub fn get_source_context(&mut self, file_path: &Path, line: u32, context_lines: u32) -> DbugResult<SourceContext> {
        // Load the file if needed
        if !self.source_cache.contains_key(file_path) {
            self.load_source_file(file_path)?;
        }
        
        // Get the lines from the cache
        let file_lines = match self.source_cache.get(file_path) {
            Some(lines) => lines,
            None => return Err(DbugError::InstrumentationError(format!("Source file not loaded: {:?}", file_path))),
        };
        
        let line_count = file_lines.len() as u32;
        
        // Calculate the range of lines to include in the context
        let start_line = line.saturating_sub(context_lines);
        let end_line = std::cmp::min(line + context_lines, line_count);
        
        let mut lines = HashMap::new();
        
        // Add lines to the context
        for line_num in start_line..=end_line {
            let idx = line_num as usize - 1; // Convert 1-indexed to 0-indexed
            if idx < file_lines.len() {
                lines.insert(line_num, file_lines[idx].clone());
            }
        }
        
        Ok(SourceContext {
            file: file_path.to_path_buf(),
            line,
            lines,
            start_line,
            end_line,
        })
    }
    
    /// Clear all mappings
    pub fn clear(&mut self) {
        self.original_to_instrumented.clear();
        self.instrumented_to_original.clear();
        self.source_cache.clear();
    }
}

/// Get a reference to the global source map
pub fn get_source_map() -> Arc<Mutex<SourceMap>> {
    SOURCE_MAP.clone()
}

/// Add a mapping to the global source map
pub fn add_mapping(
    original_file: &Path,
    original_line: u32,
    original_column: u32,
    instrumented_file: &Path,
    instrumented_line: u32,
    instrumented_column: u32,
) -> DbugResult<()> {
    let location = SourceLocation::new(
        original_file,
        original_line,
        original_column,
        instrumented_file,
        instrumented_line,
        instrumented_column,
    );
    
    let mut source_map = SOURCE_MAP.lock()
        .map_err(|_| DbugError::CommunicationError("Failed to lock source map".to_string()))?;
    
    source_map.add_mapping(location);
    Ok(())
}

/// Get a source context for a specific location
pub fn get_source_context(file_path: &Path, line: u32, context_lines: u32) -> DbugResult<SourceContext> {
    let mut source_map = SOURCE_MAP.lock()
        .map_err(|_| DbugError::CommunicationError("Failed to lock source map".to_string()))?;
    
    source_map.get_source_context(file_path, line, context_lines)
}

/// Find the original source location for an instrumented location
pub fn find_original_location(file: &str, line: u32, column: u32) -> DbugResult<Option<SourceLocation>> {
    let source_map = SOURCE_MAP.lock()
        .map_err(|_| DbugError::CommunicationError("Failed to lock source map".to_string()))?;
    
    Ok(source_map.find_original_location(file, line, column).cloned())
} 