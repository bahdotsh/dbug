use dbug::prelude::*;
use std::time::Duration;

// Remove the macro and implement the functionality directly
async fn fetch_data(id: u32) -> String {
    // Generate a unique task ID for this async execution
    let task_id = dbug::_internal::generate_async_task_id();
    
    // Notify function entry 
    dbug::_internal::enter_async_function("fetch_data", task_id);
    
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
    
    let _guard = ExitGuard { fn_name: "fetch_data", task_id };
    
    println!("Fetching data for id: {}", id);
    
    // Register the variable with the debugger
    let _ = register_var!(id);
    
    // Simulate an async operation (e.g., network request)
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Add a breakpoint here to inspect the task
    dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);
    
    let result = format!("Data for id: {}", id);
    
    // Register the result with the debugger
    let _ = register_var!(result);
    
    println!("Fetched: {}", result);
    result
}

// Remove the macro and implement the functionality directly
async fn process_data(data: String) -> String {
    // Generate a unique task ID for this async execution
    let task_id = dbug::_internal::generate_async_task_id();
    
    // Notify function entry 
    dbug::_internal::enter_async_function("process_data", task_id);
    
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
    
    let _guard = ExitGuard { fn_name: "process_data", task_id };
    
    println!("Processing data: {}", data);
    
    // Register the variable with the debugger
    let _ = register_var!(data);
    
    // Simulate some processing time
    tokio::time::sleep(Duration::from_millis(300)).await;
    
    // Add a breakpoint here to inspect the task
    dbug::_internal::async_break_point(file!(), line!(), column!(), task_id);
    
    let processed = format!("Processed: {}", data);
    
    // Register the result with the debugger
    let _ = register_var!(processed);
    
    println!("Processed result: {}", processed);
    processed
}

#[dbug]
fn main() {
    // Initialize the runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Run the async tasks
    rt.block_on(async {
        // Spawn multiple concurrent tasks
        let mut handles = Vec::new();
        
        for i in 1..=3 {
            let handle = tokio::spawn(async move {
                let data = fetch_data(i).await;
                process_data(data).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            if let Ok(result) = handle.await {
                println!("Final result: {}", result);
            }
        }
    });
    
    println!("All tasks completed!");
} 