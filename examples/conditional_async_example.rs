use dbug::prelude::*;
use std::time::Duration;

// Remove the macro and implement the functionality directly
async fn process_values(values: Vec<i32>) -> i32 {
    // Generate a unique task ID for this async execution
    let task_id = dbug::_internal::generate_async_task_id();

    // Notify function entry
    dbug::_internal::enter_async_function("process_values", task_id);

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
        fn_name: "process_values",
        task_id,
    };

    println!("Processing {} values", values.len());

    // Register the variable with the debugger
    register_var!(values);

    let mut sum = 0;
    for (index, value) in values.iter().enumerate() {
        // Only break when index is even and value is greater than 10
        // Replace async_break_when! with manual implementation
        if index % 2 == 0 && *value > 10 {
            dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);
        }

        println!("Processing value: {} at index {}", value, index);
        sum += value;

        // Simulate some async work
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("Done processing with sum: {}", sum);
    register_var!(sum);

    sum
}

#[dbug]
fn main() {
    // Initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create a vector with some values
    let values = vec![5, 15, 3, 25, 8, 12, 7, 30];

    // Run the async tasks
    let result = rt.block_on(async { process_values(values).await });

    println!("Final result: {}", result);
}
