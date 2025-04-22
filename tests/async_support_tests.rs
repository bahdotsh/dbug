//! Tests for async debugging support

use dbug::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

// Helper function to simulate async work
async fn async_work(iterations: u32, delay_ms: u64) -> u32 {
    let mut result = 0;

    for i in 0..iterations {
        result += i;
        sleep(Duration::from_millis(delay_ms)).await;
    }

    result
}

#[dbug_async]
async fn test_async_function(input: u32) -> u32 {
    // Register input variable
    register_var!(input);

    // First phase - double the input
    let doubled = input * 2;
    register_var!(doubled);

    // Add a breakpoint
    async_break_here!();

    // Do some async work
    let work_result = async_work(doubled, 10).await;
    register_var!(work_result);

    // Second phase - add 10
    let final_result = work_result + 10;
    register_var!(final_result);

    // Add another breakpoint
    async_break_here!();

    final_result
}

#[tokio::test]
async fn test_async_debugging() {
    // Test a single async function
    let result = test_async_function(5).await;

    // Calculate expected result:
    // 1. input = 5
    // 2. doubled = 5 * 2 = 10
    // 3. work_result = sum of 0 to 9 = 45
    // 4. final_result = work_result + 10 = 55
    assert_eq!(result, 55);

    // Test multiple concurrent async functions
    let mut handles = Vec::new();

    for i in 1..=3 {
        handles.push(tokio::spawn(test_async_function(i)));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        let input = i as u32 + 1;
        let doubled = input * 2;
        let work_result = (0..doubled).sum::<u32>();
        let expected = work_result + 10;
        assert_eq!(handle.await.unwrap(), expected);
    }
}

// Test the async task tracking functionality
#[tokio::test]
async fn test_async_task_tracking() {
    // Create and run several async tasks
    let mut handles = Vec::new();

    for i in 1..=5 {
        handles.push(tokio::spawn(test_async_function(i)));
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await.unwrap();
    }

    // Get all tasks and verify they're completed
    let tasks = dbug::runtime::async_support::get_all_async_tasks();

    // Check that we have at least the 5 tasks we created
    assert!(
        tasks.len() >= 5,
        "Expected at least 5 tasks, found {}",
        tasks.len()
    );

    // Count completed tasks
    let completed_count = tasks
        .iter()
        .filter(|task| task.state == dbug::runtime::async_support::AsyncTaskState::Completed)
        .count();

    // All our tasks should be completed
    assert!(
        completed_count >= 5,
        "Expected at least 5 completed tasks, found {}",
        completed_count
    );
}
