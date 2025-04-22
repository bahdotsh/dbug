// Code instrumentation module for the dbug debugger

pub mod source_mapping;

use crate::errors::{DbugError, DbugResult};
use crate::source::SourceFile;
use quote::ToTokens;
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse_file;
use syn::visit::{self, Visit};
use syn::{Attribute, File, Item, Macro, Stmt};

/// A debug point in the code
pub struct DebugPoint {
    /// The file containing the debug point
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    /// Create a new logpoint with a message
    #[allow(dead_code)]
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
            self.debug_points
                .push(DebugPoint::breakpoint(self.file_path, self.current_line));
        }

        // Check for watch macros
        if mac_str.contains("watch !") {
            let expr = mac.tokens.to_string();
            self.debug_points.push(DebugPoint::watchpoint(
                self.file_path,
                self.current_line,
                &expr,
            ));
        }
    }

    fn check_attribute(&mut self, attr: &Attribute) {
        let attr_str = attr.to_token_stream().to_string();

        // Check for #[dbug] attributes
        if attr_str.contains("dbug") {
            // For now, treat all dbug attributes as breakpoints
            // Could be refined in the future
            self.debug_points.push(DebugPoint::breakpoint(
                self.file_path,
                self.current_line + 1,
            ));
        }

        // Check for #[dbug::break_at] attributes
        if attr_str.contains("dbug :: break_at") {
            self.debug_points.push(DebugPoint::breakpoint(
                self.file_path,
                self.current_line + 1,
            ));
        }
    }
}

impl<'ast> Visit<'ast> for DebugPointVisitor<'_> {
    fn visit_macro(&mut self, mac: &'ast Macro) {
        // Try to get a reasonable line number from the name
        if let Some(name_segment) = mac.path.segments.first() {
            let name = name_segment.ident.to_string();
            if name == "break_here" {
                // Use some approximation for the line number
                self.debug_points
                    .push(DebugPoint::breakpoint(self.file_path, self.current_line));
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
    /// Track whether source mapping is enabled
    pub mapping_enabled: bool,
}

impl Instrumenter {
    /// Create a new Instrumenter
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: base_dir.to_string(),
            mapping_enabled: true,
        }
    }

    /// Enable or disable source mapping
    pub fn set_mapping_enabled(&mut self, enabled: bool) {
        self.mapping_enabled = enabled;
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
            if line_trimmed.contains("break_here!()")
                || line_trimmed.contains("dbug::break_here!()")
            {
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
            if line_trimmed.contains("watch!")
                && line_trimmed.contains("(")
                && line_trimmed.contains(")")
            {
                let start = line_trimmed.find("watch!(").map(|i| i + 7).unwrap_or(0);
                let end = line_trimmed[start..]
                    .find(")")
                    .map(|i| i + start)
                    .unwrap_or(line_trimmed.len());
                if start > 0 && end > start {
                    let expr = &line_trimmed[start..end];
                    debug_points.push(DebugPoint::watchpoint(file_path, line_num, expr));
                }
            }
        }

        debug_points
    }

    /// Instrument a file with debug points
    pub fn instrument_file(
        &self,
        file_path: &str,
        debug_points: &[DebugPoint],
    ) -> Result<(), String> {
        let original_path = Path::new(&self.base_dir).join(file_path);

        match fs::read_to_string(&original_path) {
            Ok(content) => {
                println!("Instrumenting file: {}", original_path.display());
                println!("Found {} debug points", debug_points.len());

                // For now, we don't actually modify the file
                // In a full implementation, this would insert actual instrumentation code

                // If mapping is enabled, create source mappings
                if self.mapping_enabled {
                    self.create_source_mappings(&original_path, debug_points)
                        .map_err(|e| format!("Failed to create source mappings: {}", e))?;
                }

                Ok(())
            }
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }

    /// Create source mappings for a file
    fn create_source_mappings(
        &self,
        original_path: &Path,
        debug_points: &[DebugPoint],
    ) -> DbugResult<()> {
        if !self.mapping_enabled {
            return Ok(());
        }

        // Create mappings for each debug point
        for point in debug_points {
            let line = point.line;

            // For now, we use a 1:1 mapping since we're not actually changing code
            // In a real implementation, this would map between original and instrumented locations
            source_mapping::add_mapping(
                original_path,
                line,
                0,             // column
                original_path, // instrumented file would be different in a real implementation
                line,
                0, // column
            )?;
        }

        // Also load the source file into the cache for quick access later
        let source_map = source_mapping::get_source_map();
        let mut source_map = source_map.lock().map_err(|_| {
            crate::errors::DbugError::CommunicationError("Failed to lock source map".to_string())
        })?;

        source_map.load_source_file(original_path)?;

        Ok(())
    }

    /// Get source context for a specific file and line
    pub fn get_source_context(
        &self,
        file_path: &str,
        line: u32,
        context_lines: u32,
    ) -> DbugResult<source_mapping::SourceContext> {
        let path = if Path::new(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            Path::new(&self.base_dir).join(file_path)
        };

        source_mapping::get_source_context(&path, line, context_lines)
    }

    /// Find the original source location for an instrumented location
    pub fn find_original_location(
        &self,
        file: &str,
        line: u32,
        column: u32,
    ) -> DbugResult<Option<source_mapping::SourceLocation>> {
        source_mapping::find_original_location(file, line, column)
    }

    pub fn instrument_one_file(&self, target: &Path, output: &Path) -> DbugResult<()> {
        let source_file = SourceFile::load(target.to_string_lossy().into_owned())?;
        let content = source_file.content;

        // Find debug points in the file
        let debug_points = self.find_debug_points(&target.to_string_lossy());

        // Create source mappings for the debug points
        if self.mapping_enabled && !debug_points.is_empty() {
            self.create_source_mappings(target, &debug_points)?;
        }

        // For now, just write the original content to the output file
        // In a full implementation, we would instrument the code here
        fs::write(output, content).map_err(|e| DbugError::Io(e))?;

        Ok(())
    }
}
