// Variable inspection functionality for the runtime debugger

use std::collections::HashMap;
use std::fmt;

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
}

impl fmt::Display for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableValue::Integer(i) => write!(f, "{}", i),
            VariableValue::Float(fl) => write!(f, "{}", fl),
            VariableValue::Boolean(b) => write!(f, "{}", b),
            VariableValue::String(s) => write!(f, "\"{}\"", s),
            VariableValue::Char(c) => write!(f, "'{}'", c),
            VariableValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            VariableValue::Struct(fields) => {
                write!(f, "{{")?;
                for (i, (key, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
            VariableValue::Option(opt) => match opt {
                Some(val) => write!(f, "Some({})", val),
                None => write!(f, "None"),
            },
            VariableValue::Reference(val) => write!(f, "&{}", val),
            VariableValue::Null => write!(f, "null"),
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
} 