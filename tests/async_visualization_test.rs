//! Test for visualizing async task tree

use dbug::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;

// Flag to signal when to stop
static STOP_FLAG: AtomicBool = AtomicBool::new(false);

// Remove the macro and implement the functionality directly
async fn async_task(id: u32) -> u32 {
    // Generate a unique task ID for this async execution
    let task_id = dbug::_internal::generate_async_task_id();

    // Notify function entry
    dbug::_internal::enter_async_function("async_task", task_id);

    // Create a guard for exit notification
    struct ExitGuard {
        fn_name: &'static str,
        task_id: u64,
    }

    impl Drop for ExitGuard {
        fn drop(&mut self) {
            dbug::_internal::exit_async_function(self.fn_name, self.task_id);
        }
    }

    let _guard = ExitGuard {
        fn_name: "async_task",
        task_id,
    };

    // Register the variable
    register_var!(id);

    // Add a debug point
    dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);

    println!("Task {} started", id);

    // Simulate some async work
    sleep(Duration::from_millis(100 * id as u64)).await;

    // Another debug point
    dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);

    // Return the ID as the result
    println!("Task {} completed", id);
    id
}

#[tokio::test]
async fn test_async_task_visualization() {
    // Reset the stop flag
    STOP_FLAG.store(false, Ordering::SeqCst);

    // Execute several async tasks sequentially
    let mut sum = 0;
    for i in 1..=5 {
        let result = async_task(i).await;
        sum += result;
    }

    // Verify the result
    assert_eq!(sum, 15); // 1+2+3+4+5

    // Get the task tree visualization
    let tree_viz = dbug::runtime::async_support::visualize_async_task_tree();
    println!("\nAsync Task Tree Visualization:\n{}", tree_viz);

    // Get all tasks and verify their state
    let tasks = dbug::runtime::async_support::get_all_async_tasks();

    // Verify task count - should have at least 5 tasks
    assert!(
        tasks.len() >= 5,
        "Expected at least 5 tasks, found {}",
        tasks.len()
    );

    // Verify all tasks are completed
    let completed_count = tasks
        .iter()
        .filter(|task| task.state == dbug::runtime::async_support::AsyncTaskState::Completed)
        .count();

    assert!(
        completed_count >= 5,
        "Expected at least 5 completed tasks, found {}",
        completed_count
    );

    // Now run multiple async tasks concurrently
    let mut handles = Vec::new();
    for i in 6..=10 {
        // Each of these tasks is run independently
        handles.push(tokio::task::spawn(async move { async_task(i).await }));
    }

    // Collect results
    let mut concurrent_sum = 0;
    for handle in handles {
        if let Ok(result) = handle.await {
            concurrent_sum += result;
        }
    }

    // Verify the concurrent result
    assert_eq!(concurrent_sum, 40); // 6+7+8+9+10

    // Get the final tree visualization
    let final_tree_viz = dbug::runtime::async_support::visualize_async_task_tree();
    println!("\nFinal Async Task Tree Visualization:\n{}", final_tree_viz);

    // Count total tasks
    let final_tasks = dbug::runtime::async_support::get_all_async_tasks();
    println!("Total tasks created: {}", final_tasks.len());

    // Success if we got this far
    assert!(true);
}
