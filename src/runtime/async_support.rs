use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::errors::DbugResult;
use crate::communication;

/// A unique identifier for async tasks
pub type TaskId = u64;

/// Represents the state of an async task
#[derive(Debug, Clone)]
pub struct AsyncTaskInfo {
    /// The unique identifier for this task
    pub id: TaskId,
    /// The name of the function that created this task
    pub function_name: String,
    /// The time when this task was created
    pub created_at: std::time::Instant,
    /// The current state of this task
    pub state: AsyncTaskState,
    /// The parent task that spawned this task (if any)
    pub parent_id: Option<TaskId>,
}

/// Represents the possible states of an async task
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncTaskState {
    /// The task has been created but not yet started
    Created,
    /// The task is currently running
    Running,
    /// The task is waiting for something (e.g., I/O, another task)
    Waiting,
    /// The task has been completed
    Completed,
    /// The task has been cancelled or dropped
    Cancelled,
}

// Thread-local storage for the current task ID
thread_local! {
    static CURRENT_TASK_ID: std::cell::Cell<Option<TaskId>> = std::cell::Cell::new(None);
}

// A global counter for generating unique task IDs
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

// A global registry of all active async tasks
lazy_static::lazy_static! {
    static ref ASYNC_TASK_REGISTRY: RwLock<HashMap<TaskId, AsyncTaskInfo>> = RwLock::new(HashMap::new());
}

/// Generate a new unique task ID
pub fn generate_async_task_id() -> TaskId {
    let id = NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst);
    CURRENT_TASK_ID.with(|cell| cell.set(Some(id)));
    id
}

/// Get the ID of the current async task
pub fn get_current_async_task_id() -> TaskId {
    CURRENT_TASK_ID.with(|cell| {
        cell.get().unwrap_or_else(|| {
            // If there's no task ID set, generate a new one
            let id = generate_async_task_id();
            cell.set(Some(id));
            id
        })
    })
}

/// Set the current task ID for this thread
pub fn set_current_async_task_id(id: TaskId) {
    CURRENT_TASK_ID.with(|cell| cell.set(Some(id)));
}

/// Clear the current task ID for this thread
pub fn clear_current_async_task_id() {
    CURRENT_TASK_ID.with(|cell| cell.set(None));
}

/// Register a new async task
pub fn register_async_task(function_name: &str, task_id: TaskId, parent_id: Option<TaskId>) -> DbugResult<()> {
    let task_info = AsyncTaskInfo {
        id: task_id,
        function_name: function_name.to_string(),
        created_at: std::time::Instant::now(),
        state: AsyncTaskState::Created,
        parent_id,
    };
    
    // Register the task
    let mut registry = ASYNC_TASK_REGISTRY.write().unwrap();
    registry.insert(task_id, task_info);
    
    // Notify the debugger
    communication::notify_async_task_created(function_name, task_id, parent_id)
}

/// Update the state of an async task
pub fn update_async_task_state(task_id: TaskId, state: AsyncTaskState) -> DbugResult<()> {
    let mut registry = ASYNC_TASK_REGISTRY.write().unwrap();
    
    if let Some(task) = registry.get_mut(&task_id) {
        let old_state = task.state.clone();
        task.state = state.clone();
        
        // Notify the debugger
        communication::notify_async_task_state_changed(task_id, &old_state.to_string(), &state.to_string())
    } else {
        // Task not found, might have been garbage collected
        Ok(())
    }
}

/// Mark an async task as completed
pub fn complete_async_task(task_id: TaskId) -> DbugResult<()> {
    update_async_task_state(task_id, AsyncTaskState::Completed)
}

/// Get information about an async task
pub fn get_async_task_info(task_id: TaskId) -> Option<AsyncTaskInfo> {
    let registry = ASYNC_TASK_REGISTRY.read().unwrap();
    registry.get(&task_id).cloned()
}

/// Get a list of all active async tasks
pub fn get_all_async_tasks() -> Vec<AsyncTaskInfo> {
    let registry = ASYNC_TASK_REGISTRY.read().unwrap();
    registry.values().cloned().collect()
}

/// Create a visualization of the async task tree
pub fn visualize_async_task_tree() -> String {
    let registry = ASYNC_TASK_REGISTRY.read().unwrap();
    
    // Group tasks by their parent
    let mut tree: HashMap<Option<TaskId>, Vec<&AsyncTaskInfo>> = HashMap::new();
    
    for task in registry.values() {
        tree.entry(task.parent_id).or_default().push(task);
    }
    
    // Build the visualization
    let mut result = String::new();
    result.push_str("Async Task Tree:\n");
    
    // Start with root tasks (those with no parent)
    let root_tasks = tree.get(&None).cloned().unwrap_or_default();
    
    for task in root_tasks {
        visualize_task(&tree, task, 0, &mut result);
    }
    
    result
}

/// Helper function to visualize a single task and its children
fn visualize_task(
    tree: &HashMap<Option<TaskId>, Vec<&AsyncTaskInfo>>,
    task: &AsyncTaskInfo,
    depth: usize,
    result: &mut String,
) {
    // Add indentation based on depth
    let indent = "  ".repeat(depth);
    
    // Add this task
    result.push_str(&format!(
        "{}└─ Task {} ({}): {} [{}]\n",
        indent,
        task.id,
        task.function_name,
        task.state.to_string(),
        humantime::format_duration(task.created_at.elapsed())
    ));
    
    // Add children
    if let Some(children) = tree.get(&Some(task.id)) {
        for child in children {
            visualize_task(tree, child, depth + 1, result);
        }
    }
}

// Implement ToString for AsyncTaskState
impl ToString for AsyncTaskState {
    fn to_string(&self) -> String {
        match self {
            Self::Created => "Created".to_string(),
            Self::Running => "Running".to_string(),
            Self::Waiting => "Waiting".to_string(),
            Self::Completed => "Completed".to_string(),
            Self::Cancelled => "Cancelled".to_string(),
        }
    }
} 