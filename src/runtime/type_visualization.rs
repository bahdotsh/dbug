// Type visualization functionality for complex and user-defined types
//
// This module provides utilities for visualizing complex Rust types
// in a human-readable format within the debugger.

use super::variables::{Variable, VariableValue};
use std::collections::HashMap;

/// Maximum depth for recursive type visualization
const MAX_VISUALIZATION_DEPTH: usize = 5;

/// Helper for custom type visualization
pub struct TypeVisualizer {
    /// Registry of custom visualizers by type name
    visualizers: HashMap<String, Box<dyn TypeVisualizerFn>>,
}

/// Trait for type visualizer functions
pub trait TypeVisualizerFn: Fn(&Variable) -> Option<String> + Send + Sync {}
impl<F> TypeVisualizerFn for F where F: Fn(&Variable) -> Option<String> + Send + Sync {}

impl Default for TypeVisualizer {
    fn default() -> Self {
        let mut instance = Self::new();
        instance.register_default_visualizers();
        instance
    }
}

impl TypeVisualizer {
    /// Create a new type visualizer registry
    pub fn new() -> Self {
        Self {
            visualizers: HashMap::new(),
        }
    }

    /// Register a custom visualizer for a specific type
    pub fn register_visualizer<F>(&mut self, type_name: &str, visualizer: F)
    where
        F: Fn(&Variable) -> Option<String> + Send + Sync + 'static,
    {
        self.visualizers
            .insert(type_name.to_string(), Box::new(visualizer));
    }

    /// Check if there's a visualizer for this type
    pub fn has_visualizer(&self, type_name: &str) -> bool {
        self.visualizers.contains_key(type_name)
    }

    /// Visualize a variable with a registered custom visualizer
    pub fn visualize(&self, variable: &Variable) -> Option<String> {
        if let Some(visualizer) = self.visualizers.get(&variable.type_name) {
            return visualizer(variable);
        }
        None
    }

    /// Register default visualizers for common types
    fn register_default_visualizers(&mut self) {
        // Register visualizer for Vec<T>
        self.register_visualizer("Vec", |var| {
            if let VariableValue::Vec {
                elements,
                length,
                capacity,
            } = &var.value
            {
                let mut result = format!("Vec<_> (length: {}, capacity: {})", length, capacity);
                if !elements.is_empty() {
                    result.push_str("\nContents:");
                    for (i, elem) in elements.iter().take(10).enumerate() {
                        result.push_str(&format!("\n  [{}]: {}", i, elem));
                    }
                    if elements.len() > 10 {
                        result.push_str(&format!(
                            "\n  ... and {} more elements",
                            elements.len() - 10
                        ));
                    }
                }
                return Some(result);
            }
            None
        });

        // Register visualizer for Option<T>
        self.register_visualizer("Option", |var| {
            if let VariableValue::Option(opt) = &var.value {
                match opt {
                    Some(inner) => {
                        return Some(format!("Some({})", inner));
                    }
                    None => {
                        return Some(String::from("None"));
                    }
                }
            }
            None
        });

        // Register visualizer for Result<T, E>
        self.register_visualizer("Result", |var| {
            if let VariableValue::Complex { fields, .. } = &var.value {
                if fields.contains_key("Ok") {
                    if let Some(ok_value) = fields.get("Ok") {
                        return Some(format!("Ok({})", ok_value));
                    }
                } else if fields.contains_key("Err") {
                    if let Some(err_value) = fields.get("Err") {
                        return Some(format!("Err({})", err_value));
                    }
                }
            }
            None
        });

        // Register visualizer for String
        self.register_visualizer("String", |var| {
            if let VariableValue::String(s) = &var.value {
                let mut result = format!("String (length: {})", s.len());
                if !s.is_empty() {
                    result.push_str(&format!("\nContents: \"{}\"", s));

                    // Add special character visualization
                    let mut special_chars = false;
                    let mut special_info = String::new();

                    for (i, c) in s.chars().enumerate() {
                        if c < ' ' || c > '~' {
                            if !special_chars {
                                special_chars = true;
                                special_info.push_str("\nSpecial characters:");
                            }
                            special_info.push_str(&format!(
                                "\n  [{}]: '{}' (Unicode: U+{:04X})",
                                i,
                                c.escape_unicode(),
                                c as u32
                            ));
                        }
                    }

                    if special_chars {
                        result.push_str(&special_info);
                    }
                }
                return Some(result);
            }
            None
        });

        // Register visualizer for HashMap
        self.register_visualizer("HashMap", |var| {
            if let VariableValue::HashMap {
                entries,
                size,
                capacity,
            } = &var.value
            {
                let mut result = format!("HashMap (size: {}, capacity: {})", size, capacity);
                if !entries.is_empty() {
                    result.push_str("\nEntries:");
                    for (i, (key, value)) in entries.iter().take(10).enumerate() {
                        result.push_str(&format!("\n  {}: {} => {}", i, key, value));
                    }
                    if entries.len() > 10 {
                        result
                            .push_str(&format!("\n  ... and {} more entries", entries.len() - 10));
                    }
                }
                return Some(result);
            }
            None
        });
    }

    /// Create a visualization for any composite type
    pub fn create_composite_visualization(
        &self,
        type_name: &str,
        var: &Variable,
        depth: usize,
    ) -> String {
        if depth > MAX_VISUALIZATION_DEPTH {
            return format!("{}... (max depth reached)", type_name);
        }

        // Try registered visualizer first
        if let Some(visualization) = self.visualize(var) {
            return visualization;
        }

        // Fallback visualization logic based on value type
        match &var.value {
            VariableValue::Struct(fields) => {
                let mut result = format!("{} {{", type_name);
                for (name, value) in fields {
                    result.push_str(&format!("\n  {}: {}", name, value));
                }
                result.push_str("\n}");
                result
            }
            VariableValue::Complex {
                summary,
                fields,
                children,
                ..
            } => {
                let mut result = format!("{} {{{}}}", type_name, summary);

                if !fields.is_empty() {
                    result.push_str("\nFields:");
                    for (name, value) in fields {
                        result.push_str(&format!("\n  {}: {}", name, value));
                    }
                }

                if let Some(elements) = children {
                    if !elements.is_empty() {
                        result.push_str("\nElements:");
                        for (i, value) in elements.iter().take(10).enumerate() {
                            result.push_str(&format!("\n  [{}]: {}", i, value));
                        }

                        if elements.len() > 10 {
                            result.push_str(&format!(
                                "\n  ... and {} more elements",
                                elements.len() - 10
                            ));
                        }
                    }
                }

                result
            }
            // Fallback for other types
            _ => format!("{} = {}", type_name, var.value),
        }
    }
}
