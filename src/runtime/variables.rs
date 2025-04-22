// Variable inspection functionality for the runtime debugger

use std::collections::HashMap;
use std::fmt;

/// Maximum depth for recursive data structure visualization
pub const MAX_VISUALIZATION_DEPTH: usize = 3;

/// Represents the value of a variable
#[derive(Debug, Clone)]
pub enum VariableValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    #[allow(dead_code)]
    Array(Vec<VariableValue>),
    #[allow(dead_code)]
    Struct(HashMap<String, VariableValue>),
    #[allow(dead_code)]
    Option(Option<Box<VariableValue>>),
    #[allow(dead_code)]
    Reference(Box<VariableValue>),
    #[allow(dead_code)]
    Null,
    /// Enhanced type for complex data structures
    Complex {
        /// Type name of the complex structure (like Vec, HashMap, etc.)
        type_name: String,
        /// Summary representation of the structure
        summary: String,
        /// Detailed fields of the structure
        fields: HashMap<String, VariableValue>,
        /// Child elements if applicable (for collections)
        children: Option<Vec<VariableValue>>,
    },
    /// Vector-specific representation for better visualization
    Vec {
        /// Elements in the vector
        elements: Vec<VariableValue>,
        /// Length of the vector
        length: usize,
        /// Capacity of the vector
        capacity: usize,
    },
    /// HashMap-specific representation
    HashMap {
        /// Key-value pairs
        entries: Vec<(VariableValue, VariableValue)>,
        /// Number of entries
        size: usize,
        /// Capacity of the HashMap
        capacity: usize,
    },
}

impl fmt::Display for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_depth(f, 0)
    }
}

impl VariableValue {
    /// Format the value with a depth limit to avoid deep recursion
    fn fmt_with_depth(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        // If we've reached the maximum depth, show a placeholder
        if depth > MAX_VISUALIZATION_DEPTH {
            return write!(f, "...");
        }

        match self {
            VariableValue::Integer(i) => write!(f, "{}", i),
            VariableValue::Float(fl) => write!(f, "{}", fl),
            VariableValue::Boolean(b) => write!(f, "{}", b),
            VariableValue::String(s) => write!(f, "\"{}\"", s),
            VariableValue::Char(c) => write!(f, "'{}'", c),
            VariableValue::Array(arr) => {
                write!(f, "[")?;
                if !arr.is_empty() && depth == MAX_VISUALIZATION_DEPTH {
                    write!(f, "...({})", arr.len())?;
                } else {
                    for (i, val) in arr.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        val.fmt_with_depth(f, depth + 1)?;
                    }
                }
                write!(f, "]")
            }
            VariableValue::Struct(fields) => {
                write!(f, "{{")?;
                if !fields.is_empty() && depth == MAX_VISUALIZATION_DEPTH {
                    write!(f, "...({})", fields.len())?;
                } else {
                    let mut first = true;
                    for (key, val) in fields {
                        if !first {
                            write!(f, ", ")?;
                        }
                        first = false;
                        write!(f, "{}: ", key)?;
                        val.fmt_with_depth(f, depth + 1)?;
                    }
                }
                write!(f, "}}")
            }
            VariableValue::Option(opt) => match opt {
                Some(val) => {
                    write!(f, "Some(")?;
                    val.fmt_with_depth(f, depth + 1)?;
                    write!(f, ")")
                }
                None => write!(f, "None"),
            },
            VariableValue::Reference(val) => {
                write!(f, "&")?;
                val.fmt_with_depth(f, depth + 1)
            }
            VariableValue::Null => write!(f, "null"),
            VariableValue::Complex {
                type_name,
                summary,
                fields,
                children,
            } => {
                write!(f, "{}{{ {} }}", type_name, summary)?;

                if depth < MAX_VISUALIZATION_DEPTH {
                    if !fields.is_empty() {
                        write!(f, " {{")?;
                        let mut first = true;
                        for (key, val) in fields {
                            if !first {
                                write!(f, ", ")?;
                            }
                            first = false;
                            write!(f, "{}: ", key)?;
                            val.fmt_with_depth(f, depth + 1)?;
                        }
                        write!(f, "}}")?;
                    }

                    if let Some(elements) = children {
                        if !elements.is_empty() {
                            write!(f, " [")?;
                            for (i, val) in elements.iter().enumerate() {
                                if i > 0 {
                                    write!(f, ", ")?;
                                }
                                if i >= 10 && elements.len() > 12 {
                                    write!(f, "... ({} more)", elements.len() - i)?;
                                    break;
                                }
                                val.fmt_with_depth(f, depth + 1)?;
                            }
                            write!(f, "]")?;
                        }
                    }
                }

                Ok(())
            }
            VariableValue::Vec {
                elements,
                length,
                capacity,
            } => {
                write!(f, "Vec (len: {}, capacity: {}) [", length, capacity)?;

                if depth < MAX_VISUALIZATION_DEPTH {
                    for (i, val) in elements.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        if i >= 10 && elements.len() > 12 {
                            write!(f, "... ({} more)", elements.len() - i)?;
                            break;
                        }
                        val.fmt_with_depth(f, depth + 1)?;
                    }
                } else {
                    write!(f, "...")?;
                }

                write!(f, "]")
            }
            VariableValue::HashMap {
                entries,
                size,
                capacity,
            } => {
                write!(f, "HashMap (size: {}, capacity: {}) {{", size, capacity)?;

                if depth < MAX_VISUALIZATION_DEPTH {
                    for (i, (key, val)) in entries.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        if i >= 10 && entries.len() > 12 {
                            write!(f, "... ({} more)", entries.len() - i)?;
                            break;
                        }
                        key.fmt_with_depth(f, depth + 1)?;
                        write!(f, ": ")?;
                        val.fmt_with_depth(f, depth + 1)?;
                    }
                } else {
                    write!(f, "...")?;
                }

                write!(f, "}}")
            }
        }
    }

    /// Create a vector representation
    #[allow(dead_code)]
    pub fn new_vec(elements: Vec<VariableValue>, capacity: usize) -> Self {
        let length = elements.len();
        VariableValue::Vec {
            elements,
            length,
            capacity,
        }
    }

    /// Create a hashmap representation
    #[allow(dead_code)]
    pub fn new_hashmap(entries: Vec<(VariableValue, VariableValue)>, capacity: usize) -> Self {
        let size = entries.len();
        VariableValue::HashMap {
            entries,
            size,
            capacity,
        }
    }

    /// Create a complex structure representation
    #[allow(dead_code)]
    pub fn new_complex(
        type_name: &str,
        summary: &str,
        fields: HashMap<String, VariableValue>,
        children: Option<Vec<VariableValue>>,
    ) -> Self {
        VariableValue::Complex {
            type_name: type_name.to_string(),
            summary: summary.to_string(),
            fields,
            children,
        }
    }
}

/// Variable change status to track modifications
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    /// Variable is unchanged
    Unchanged,
    /// Variable is newly created
    New,
    /// Variable value has changed
    Modified,
    /// A child of this variable has changed (for complex types)
    ChildModified,
}

/// A variable in the debugged program
#[derive(Debug, Clone)]
pub struct Variable {
    /// The name of the variable
    pub name: String,
    /// The type of the variable
    pub type_name: String,
    /// The value of the variable
    pub value: VariableValue,
    /// The scope level (useful for determining variable shadowing)
    pub scope_level: u32,
    /// Whether the variable is mutable
    pub is_mutable: bool,
    /// Previous value for change detection
    pub previous_value: Option<VariableValue>,
    /// Change status of this variable
    pub change_status: ChangeStatus,
    /// Last update timestamp
    pub last_updated: std::time::Instant,
}

impl Variable {
    /// Create a new variable
    pub fn new(
        name: &str,
        type_name: &str,
        value: VariableValue,
        scope_level: u32,
        is_mutable: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            value,
            scope_level,
            is_mutable,
            previous_value: None,
            change_status: ChangeStatus::New,
            last_updated: std::time::Instant::now(),
        }
    }

    /// Update the value of the variable
    pub fn update_value(&mut self, new_value: VariableValue) {
        // Store the previous value
        self.previous_value = Some(self.value.clone());
        // Update the value
        self.value = new_value;
        // Set the change status
        self.change_status = ChangeStatus::Modified;
        // Update the timestamp
        self.last_updated = std::time::Instant::now();
    }

    /// Reset the change status
    pub fn reset_change_status(&mut self) {
        self.change_status = ChangeStatus::Unchanged;
    }

    /// Check if the variable has changed
    pub fn has_changed(&self) -> bool {
        self.change_status != ChangeStatus::Unchanged
    }

    /// Get the time since last update
    pub fn time_since_update(&self) -> std::time::Duration {
        self.last_updated.elapsed()
    }
}

/// Represents the variable inspector that tracks variables during debugging
#[derive(Clone)]
pub struct VariableInspector {
    /// Currently visible variables
    variables: HashMap<String, Variable>,
    /// Current scope level
    current_scope: u32,
    /// Variables that have changed in the last update
    changed_variables: Vec<String>,
}

impl Default for VariableInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableInspector {
    /// Create a new VariableInspector
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_scope: 0,
            changed_variables: Vec::new(),
        }
    }

    /// Register a new variable
    pub fn register_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    /// Get a variable by name
    pub fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }

    /// Enter a new scope
    #[allow(dead_code)]
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
    }

    /// Exit the current scope, removing variables at that scope
    #[allow(dead_code)]
    pub fn exit_scope(&mut self) {
        // Remove variables at the current scope level
        self.variables
            .retain(|_, var| var.scope_level < self.current_scope);
        self.current_scope = self.current_scope.saturating_sub(1);
    }

    /// Check if two variable values are equal (helper method)
    fn are_values_equal(val1: &VariableValue, val2: &VariableValue) -> bool {
        match (val1, val2) {
            (VariableValue::Integer(i1), VariableValue::Integer(i2)) => i1 == i2,
            (VariableValue::Float(f1), VariableValue::Float(f2)) => {
                (f1 - f2).abs() < std::f64::EPSILON
            }
            (VariableValue::Boolean(b1), VariableValue::Boolean(b2)) => b1 == b2,
            (VariableValue::String(s1), VariableValue::String(s2)) => s1 == s2,
            (VariableValue::Char(c1), VariableValue::Char(c2)) => c1 == c2,
            (VariableValue::Null, VariableValue::Null) => true,
            // For complex types, we just do a reference comparison
            // A more sophisticated implementation would recursively compare structure
            _ => false,
        }
    }

    /// Update a variable or register a new one if it doesn't exist
    pub fn update_variable(&mut self, name: &str, value: VariableValue) -> Result<(), String> {
        if let Some(var) = self.variables.get_mut(name) {
            // Only update if mutable or changing to the same value (for immutable vars)
            if var.is_mutable || !Self::are_values_equal(&var.value, &value) {
                var.update_value(value);
                self.changed_variables.push(name.to_string());
            }
            Ok(())
        } else {
            Err(format!("Variable {} not found", name))
        }
    }

    /// Compare two variable values for equality
    fn values_equal(&self, val1: &VariableValue, val2: &VariableValue) -> bool {
        Self::are_values_equal(val1, val2)
    }

    /// Get all variables currently in scope
    pub fn get_all_variables(&self) -> Vec<&Variable> {
        self.variables.values().collect()
    }

    /// Create a detailed visualization of a variable
    #[allow(dead_code)]
    pub fn visualize_variable(&self, name: &str) -> Option<String> {
        let var = self.get_variable(name)?;
        Some(self.create_detailed_visualization(var))
    }

    /// Creates detailed visualization of a variable with better type support
    fn create_detailed_visualization(&self, var: &Variable) -> String {
        let mut result = format!("{}: {} = {}", var.name, var.type_name, var.value);

        // Add additional type-specific details
        match &var.value {
            VariableValue::Vec {
                elements,
                length,
                capacity,
            } => {
                result.push_str(&format!("\n  Length: {}", length));
                result.push_str(&format!("\n  Capacity: {}", capacity));

                // Show first few elements with indices
                if !elements.is_empty() {
                    result.push_str("\n  Elements:");
                    for (i, element) in elements.iter().take(10).enumerate() {
                        result.push_str(&format!("\n    [{}]: {}", i, element));
                    }

                    if elements.len() > 10 {
                        result.push_str(&format!(
                            "\n    ... and {} more elements",
                            elements.len() - 10
                        ));
                    }
                }
            }
            VariableValue::HashMap {
                entries,
                size,
                capacity,
            } => {
                result.push_str(&format!("\n  Size: {}", size));
                result.push_str(&format!("\n  Capacity: {}", capacity));

                // Show key-value pairs
                if !entries.is_empty() {
                    result.push_str("\n  Entries:");
                    for (i, (key, value)) in entries.iter().take(10).enumerate() {
                        result.push_str(&format!("\n    {}: {} => {}", i, key, value));
                    }

                    if entries.len() > 10 {
                        result.push_str(&format!(
                            "\n    ... and {} more entries",
                            entries.len() - 10
                        ));
                    }
                }
            }
            VariableValue::Struct(fields) => {
                if !fields.is_empty() {
                    result.push_str("\n  Fields:");
                    for (name, value) in fields {
                        result.push_str(&format!("\n    {}: {}", name, value));
                    }
                }
            }
            VariableValue::Complex {
                type_name,
                summary,
                fields,
                children,
            } => {
                result.push_str(&format!("\n  Type: {}", type_name));
                result.push_str(&format!("\n  Summary: {}", summary));

                if !fields.is_empty() {
                    result.push_str("\n  Fields:");
                    for (name, value) in fields {
                        result.push_str(&format!("\n    {}: {}", name, value));
                    }
                }

                if let Some(elements) = children {
                    if !elements.is_empty() {
                        result.push_str("\n  Elements:");
                        for (i, element) in elements.iter().take(10).enumerate() {
                            result.push_str(&format!("\n    [{}]: {}", i, element));
                        }

                        if elements.len() > 10 {
                            result.push_str(&format!(
                                "\n    ... and {} more elements",
                                elements.len() - 10
                            ));
                        }
                    }
                }
            }
            VariableValue::Option(opt) => match opt {
                Some(value) => {
                    result.push_str("\n  Contains value:");
                    result.push_str(&format!("\n    {}", value));
                }
                None => {
                    result.push_str("\n  Contains no value (None)");
                }
            },
            VariableValue::Reference(value) => {
                result.push_str("\n  Reference to:");
                result.push_str(&format!("\n    {}", value));
            }
            VariableValue::Array(elements) => {
                result.push_str(&format!("\n  Length: {}", elements.len()));

                // Show elements with indices
                if !elements.is_empty() {
                    result.push_str("\n  Elements:");
                    for (i, element) in elements.iter().take(10).enumerate() {
                        result.push_str(&format!("\n    [{}]: {}", i, element));
                    }

                    if elements.len() > 10 {
                        result.push_str(&format!(
                            "\n    ... and {} more elements",
                            elements.len() - 10
                        ));
                    }
                }
            }
            // For primitive types, the default representation is sufficient
            _ => {}
        }

        result
    }

    /// Get all variables that have changed since last check
    pub fn get_changed_variables(&mut self) -> Vec<&Variable> {
        let changed = self.changed_variables.clone();
        self.changed_variables.clear();

        // Get the variables from their names
        changed
            .iter()
            .filter_map(|name| self.variables.get(name))
            .collect()
    }

    /// Reset all change statuses
    pub fn reset_change_status(&mut self) {
        for var in self.variables.values_mut() {
            var.reset_change_status();
        }
        self.changed_variables.clear();
    }
}
