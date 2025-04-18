// Code instrumentation module for the dbug debugger

use std::fs;
use std::path::Path;
use syn::{File, Item, Expr, Stmt, Attribute, Macro, Meta};
use syn::visit::{self, Visit};
use syn::parse_file;
use quote::ToTokens;

/// A debug point in the code
pub struct DebugPoint {
    /// The file containing the debug point
    pub file: String,
    /// The line number of the debug point
    pub line: u32,
    /// The type of debug point
    pub point_type: DebugPointType,
}

/// The type of debug point
pub enum DebugPointType {
    /// A breakpoint that pauses execution
    Breakpoint,
    /// A watch point that displays a value
    Watchpoint(String),
    /// A log point that prints a message
    LogPoint(String),
}

impl DebugPoint {
    /// Create a new breakpoint debug point
    pub fn breakpoint(file: &str, line: u32) -> Self {
        Self {
            file: file.to_string(),
            line,
            point_type: DebugPointType::Breakpoint,
        }
    }
    
    /// Create a new watchpoint debug point
    pub fn watchpoint(file: &str, line: u32, expression: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            point_type: DebugPointType::Watchpoint(expression.to_string()),
        }
    }
    
    /// Create a new logpoint debug point
    pub fn logpoint(file: &str, line: u32, message: &str) -> Self {
        Self {
            file: file.to_string(),
            line,
            point_type: DebugPointType::LogPoint(message.to_string()),
        }
    }
}

/// Debug point visitor for AST traversal
struct DebugPointVisitor<'a> {
    file_path: &'a str,
    debug_points: Vec<DebugPoint>,
    current_line: u32,
}

impl<'a> DebugPointVisitor<'a> {
    fn new(file_path: &'a str) -> Self {
        Self {
            file_path,
            debug_points: Vec::new(),
            current_line: 1,
        }
    }
    
    // Helper methods to detect debug points
    fn check_macro(&mut self, mac: &Macro) {
        let mac_str = mac.to_token_stream().to_string();
        
        // Check for break_here! macro calls
        if mac_str.contains("break_here !") {
            // Get a reasonable line number - this is a simplification
            self.debug_points.push(DebugPoint::breakpoint(self.file_path, self.current_line));
        }
        
        // Check for watch macros
        if mac_str.contains("watch !") {
            let expr = mac.tokens.to_string();
            self.debug_points.push(DebugPoint::watchpoint(self.file_path, self.current_line, &expr));
        }
    }
    
    fn check_attribute(&mut self, attr: &Attribute) {
        let attr_str = attr.to_token_stream().to_string();
        
        // Check for #[dbug] attributes
        if attr_str.contains("dbug") {
            // For now, treat all dbug attributes as breakpoints
            // Could be refined in the future
            self.debug_points.push(DebugPoint::breakpoint(self.file_path, self.current_line + 1));
        }
        
        // Check for #[dbug::break_at] attributes
        if attr_str.contains("dbug :: break_at") {
            self.debug_points.push(DebugPoint::breakpoint(self.file_path, self.current_line + 1));
        }
    }
}

impl<'ast, 'a> Visit<'ast> for DebugPointVisitor<'a> {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        // Try to get a reasonable line number from the name
        if let Some(name_segment) = mac.path.segments.first() {
            let name = name_segment.ident.to_string();
            if name == "break_here" {
                // Use some approximation for the line number
                self.debug_points.push(DebugPoint::breakpoint(self.file_path, self.current_line));
            }
        }
        
        self.check_macro(mac);
        visit::visit_macro(self, mac);
    }
    
    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        self.check_attribute(attr);
        visit::visit_attribute(self, attr);
    }
    
    fn visit_file(&mut self, file: &'ast File) {
        // Visit all items in the file
        for item in &file.items {
            match item {
                Item::Fn(item_fn) => {
                    // Check function attributes
                    for attr in &item_fn.attrs {
                        self.check_attribute(attr);
                    }
                    
                    // Continue traversal
                    visit::visit_item_fn(self, item_fn);
                }
                _ => visit::visit_item(self, item),
            }
        }
    }
    
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        // When we encounter a statement, we update our line tracking
        // This is a simplification, but helps us track location
        self.current_line += 1;
        visit::visit_stmt(self, stmt);
    }
}

/// Instrumenter for adding debug points to Rust code
pub struct Instrumenter {
    /// The base directory for resolving relative paths
    pub base_dir: String,
}

impl Instrumenter {
    /// Create a new Instrumenter
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: base_dir.to_string(),
        }
    }
    
    /// Find all debug points in a file
    pub fn find_debug_points(&self, file_path: &str) -> Vec<DebugPoint> {
        let path = Path::new(&self.base_dir).join(file_path);
        
        // Try the advanced parser first
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(file_ast) = parse_file(&content) {
                let mut visitor = DebugPointVisitor::new(file_path);
                visit::visit_file(&mut visitor, &file_ast);
                
                // If we found debug points, return them
                if !visitor.debug_points.is_empty() {
                    return visitor.debug_points;
                }
            }
        }
        
        // Fallback to the simpler approach if the advanced parser fails
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(), // If we can't read the file, return empty
        };
        
        // Parse the file to find debug points
        let mut debug_points = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i as u32 + 1;
            let line_trimmed = line.trim();
            
            // Check for break_here macro calls
            if line_trimmed.contains("break_here!()") || line_trimmed.contains("dbug::break_here!()") {
                debug_points.push(DebugPoint::breakpoint(file_path, line_num));
            }
            
            // Check for attribute macros like #[dbug::break_at]
            if line_trimmed.contains("#[dbug::break_at]") {
                // The breakpoint is on the next line
                if i + 1 < lines.len() {
                    debug_points.push(DebugPoint::breakpoint(file_path, line_num + 1));
                }
            }
            
            // Look for dbug functions
            if line_trimmed.contains("#[dbug]") {
                // Find the function name in the next line
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if next_line.starts_with("fn ") {
                        // This is a function with dbug attribute
                        // We might want to track these separately in the future
                    }
                }
            }
            
            // Check for watch expressions (not yet implemented in the macros, but preparing for future)
            if line_trimmed.contains("watch!") && line_trimmed.contains("(") && line_trimmed.contains(")") {
                let start = line_trimmed.find("watch!(").map(|i| i + 7).unwrap_or(0);
                let end = line_trimmed[start..].find(")").map(|i| i + start).unwrap_or(line_trimmed.len());
                if start > 0 && end > start {
                    let expr = &line_trimmed[start..end];
                    debug_points.push(DebugPoint::watchpoint(file_path, line_num, expr));
                }
            }
        }
        
        debug_points
    }
    
    /// Instrument a file with debug points
    pub fn instrument_file(&self, file_path: &str, debug_points: &[DebugPoint]) -> Result<(), String> {
        let path = Path::new(&self.base_dir).join(file_path);
        
        match fs::read_to_string(&path) {
            Ok(_content) => {
                println!("Instrumenting file: {}", path.display());
                println!("Found {} debug points", debug_points.len());
                
                for point in debug_points {
                    match &point.point_type {
                        DebugPointType::Breakpoint => {
                            println!("  Breakpoint at line {}", point.line);
                        }
                        DebugPointType::Watchpoint(expr) => {
                            println!("  Watchpoint at line {}: watch {}", point.line, expr);
                        }
                        DebugPointType::LogPoint(msg) => {
                            println!("  Logpoint at line {}: {}", point.line, msg);
                        }
                    }
                }
                
                // Future: Actually modify the file to insert instrumentation code
                // For now, just report on the debug points
                
                Ok(())
            }
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }
} 