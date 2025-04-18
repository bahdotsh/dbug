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
    Array(Vec<VariableValue>),
    Struct(HashMap<String, VariableValue>),
    Option(Option<Box<VariableValue>>),
    Reference(Box<VariableValue>),
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
            VariableValue::Complex { type_name, summary, fields, children } => {
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
            VariableValue::Vec { elements, length, capacity } => {
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
            VariableValue::HashMap { entries, size, capacity } => {
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
    pub fn new_vec(elements: Vec<VariableValue>, capacity: usize) -> Self {
        let length = elements.len();
        VariableValue::Vec {
            elements,
            length,
            capacity,
        }
    }
    
    /// Create a hashmap representation
    pub fn new_hashmap(entries: Vec<(VariableValue, VariableValue)>, capacity: usize) -> Self {
        let size = entries.len();
        VariableValue::HashMap {
            entries,
            size,
            capacity,
        }
    }
    
    /// Create a complex structure representation
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

/// A variable in the debugged program
#[derive(Debug, Clone)]
pub struct Variable {
    /// The name of the variable
    pub name: String,
    /// The type of the variable
    pub type_name: String,
    /// The current value of the variable
    pub value: VariableValue,
    /// The scope level (useful for determining variable shadowing)
    pub scope_level: u32,
    /// Whether the variable is mutable
    pub is_mutable: bool,
}

impl Variable {
    /// Create a new variable
    pub fn new(name: &str, type_name: &str, value: VariableValue, scope_level: u32, is_mutable: bool) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
            value,
            scope_level,
            is_mutable,
        }
    }
}

/// Handles variable inspection and manipulation
pub struct VariableInspector {
    /// Currently visible variables
    variables: HashMap<String, Variable>,
    /// Current scope level
    current_scope: u32,
}

impl VariableInspector {
    /// Create a new VariableInspector
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_scope: 0,
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
    pub fn enter_scope(&mut self) {
        self.current_scope += 1;
    }

    /// Exit the current scope, removing variables at that scope
    pub fn exit_scope(&mut self) {
        // Remove variables at the current scope level
        self.variables.retain(|_, var| var.scope_level < self.current_scope);
        self.current_scope = self.current_scope.saturating_sub(1);
    }

    /// Update a variable's value
    pub fn update_variable(&mut self, name: &str, value: VariableValue) -> Result<(), String> {
        if let Some(var) = self.variables.get_mut(name) {
            if var.is_mutable {
                var.value = value;
                Ok(())
            } else {
                Err(format!("Cannot modify immutable variable '{}'", name))
            }
        } else {
            Err(format!("Variable '{}' not found", name))
        }
    }

    /// Get all variables currently in scope
    pub fn get_all_variables(&self) -> Vec<&Variable> {
        self.variables.values().collect()
    }
    
    /// Create a detailed visualization of a variable
    pub fn visualize_variable(&self, name: &str) -> Option<String> {
        let var = self.get_variable(name)?;
        Some(self.create_detailed_visualization(var))
    }
    
    /// Create a detailed visualization of a variable
    fn create_detailed_visualization(&self, var: &Variable) -> String {
        let mut result = format!("Variable: {} ({})\n", var.name, var.type_name);
        result.push_str(&format!("Mutability: {}\n", if var.is_mutable { "mutable" } else { "immutable" }));
        result.push_str("Value: ");
        
        match &var.value {
            VariableValue::Complex { type_name, summary, fields, children } => {
                result.push_str(&format!("{} {{\n", type_name));
                result.push_str(&format!("  Summary: {}\n", summary));
                
                if !fields.is_empty() {
                    result.push_str("  Fields:\n");
                    for (name, value) in fields {
                        result.push_str(&format!("    {}: {}\n", name, value));
                    }
                }
                
                if let Some(elems) = children {
                    if !elems.is_empty() {
                        result.push_str(&format!("  Elements ({}):\n", elems.len()));
                        for (i, elem) in elems.iter().enumerate() {
                            if i >= 10 && elems.len() > 12 {
                                result.push_str(&format!("    ... ({} more elements)\n", elems.len() - i));
                                break;
                            }
                            result.push_str(&format!("    [{}]: {}\n", i, elem));
                        }
                    }
                }
                
                result.push_str("}");
            }
            VariableValue::Vec { elements, length, capacity } => {
                result.push_str(&format!("Vec<{}> (len: {}, capacity: {})\n", 
                    if !elements.is_empty() {
                        let elem_type = match &elements[0] {
                            VariableValue::Integer(_) => "i64",
                            VariableValue::Float(_) => "f64",
                            VariableValue::Boolean(_) => "bool",
                            VariableValue::String(_) => "String",
                            VariableValue::Char(_) => "char",
                            _ => "T",
                        };
                        elem_type
                    } else {
                        "T"
                    },
                    length, capacity));
                
                for (i, elem) in elements.iter().enumerate() {
                    if i >= 15 && elements.len() > 17 {
                        result.push_str(&format!("  ... ({} more elements)\n", elements.len() - i));
                        break;
                    }
                    result.push_str(&format!("  [{}]: {}\n", i, elem));
                }
            }
            VariableValue::HashMap { entries, size, capacity } => {
                result.push_str(&format!("HashMap (size: {}, capacity: {})\n", size, capacity));
                
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i >= 15 && entries.len() > 17 {
                        result.push_str(&format!("  ... ({} more entries)\n", entries.len() - i));
                        break;
                    }
                    result.push_str(&format!("  {}: {}\n", key, value));
                }
            }
            VariableValue::Struct(fields) => {
                result.push_str("{\n");
                for (name, value) in fields {
                    result.push_str(&format!("  {}: {}\n", name, value));
                }
                result.push_str("}");
            }
            _ => {
                result.push_str(&format!("{}", var.value));
            }
        }
        
        result
    }
} 