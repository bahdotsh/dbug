//! Prelude module that re-exports commonly used items

// Re-export the macros from the proc-macro crate
pub use dbug_macros::dbug;
pub use dbug_macros::dbug_async;
pub use dbug_macros::break_here;
pub use dbug_macros::async_break_here;
pub use dbug_macros::async_break_when;
pub use dbug_macros::break_at;
pub use dbug_macros::register_var;

// Re-export runtime types that might be useful in user code
pub use crate::runtime::{Variable, VariableValue};
pub use crate::runtime::async_support::{TaskId, AsyncTaskInfo, AsyncTaskState};

// Re-export the register_variable function
pub use crate::_internal::register_variable;

// Re-export error types and utilities
pub use crate::errors::{DbugError, DbugResult, ErrorExt}; 