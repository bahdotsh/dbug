# Dbug

A CLI-based debugger for Rust projects that makes debugging simple and intuitive.

## Overview

Dbug is designed to make the debugging experience for Rust developers as straightforward as possible. By allowing users to place debug points directly in their code and providing a simple CLI interface, Dbug eliminates the complexity typically associated with debugging tools.

## Features

- **Simple debug points** - Add breakpoints with minimal syntax
- **Intuitive CLI interface** - No complex commands to remember
- **Value inspection** - Easily view variables and their values
- **Flow control** - Step through code execution with simple commands
- **Conditional breakpoints** - Break only when specific conditions are met
- **Watch expressions** - Monitor variables as they change
- **Async debugging** - Support for debugging async Rust code
- **Memory-mapped communication** - Efficient IPC between debugger and debuggee

## Installation

```bash
# Not yet available via cargo install
# Coming soon!

# For now, clone and build from source
git clone https://github.com/yourusername/dbug.git
cd dbug
cargo install --path .
```

## Usage

```bash
# Build a project with debug instrumentation
dbug build /path/to/rust/project

# Build and run a project with the debugger
dbug run /path/to/rust/project

# Display help
dbug help

# Display version
dbug version
```

## Adding Debug Points

Add debug points to your code using the `dbug` macro:

```rust
use dbug::prelude::*;

#[dbug]
fn some_function() {
    let x = 42;
    dbug::break_here!(); // A debug point
    let y = x + 1;
    // ...
}
```

Or use inline annotations:

```rust
fn some_function() {
    let x = 42;
    #[dbug::break]
    let y = x + 1;
    // ...
}
```

## Debugger Commands

When a debug point is hit, you can use the following commands:

- `n` or `next`: Step to the next line
- `s` or `step`: Step into a function call
- `o` or `out`: Step out of the current function
- `c` or `continue`: Continue execution until the next breakpoint
- `p <expression>` or `print <expression>`: Print the value of an expression
- `w <expression>` or `watch <expression>`: Watch an expression for changes
- `b <file:line>` or `break <file:line>`: Add a new breakpoint
- `q` or `quit`: Quit the debugger

## Conditional Breakpoints

You can add conditional breakpoints to break only when specific conditions are met:

```rust
use dbug::prelude::*;

#[dbug]
fn process_items(items: Vec<i32>) {
    for (index, item) in items.iter().enumerate() {
        // This will break only when index is greater than 5 and item is divisible by 3
        dbug::break_if!(index > 5 && item % 3 == 0);
        
        // Process item...
    }
}
```

You can also configure breakpoints via the debugger interface:

```
# Add a breakpoint at line 42 of main.rs
b main.rs:42

# Add a conditional breakpoint
b main.rs:30 if x > 10

# Add a hit count breakpoint (break on the 5th hit)
b main.rs:25 count=5

# Add a combined condition (break when x > 10 on the 3rd hit)
b main.rs:20 if x > 10 count=3
```

## Debugging Async Rust Code

Dbug provides special support for debugging async Rust code:

```rust
use dbug::prelude::*;

#[dbug_async]
async fn fetch_data(url: &str) -> Result<String, Error> {
    // This will break during async execution
    dbug::async_break_here!();
    
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    
    Ok(body)
}
```

When debugging async code, Dbug keeps track of task IDs and provides details about the async execution context:

```
[DBUG] Async task created: fetch_data (task_id: 42, parent: None)
[DBUG] Async function entered: fetch_data (task_id: 42)
[DBUG] Async breakpoint hit: src/main.rs:8:5 in fetch_data (task_id: 42)
> src/main.rs:8
  |
7 |     // This will break during async execution
8 |     dbug::async_break_here!();
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^
9 |     
```

## Using the TUI Interface

Dbug provides a Terminal User Interface (TUI) mode for a richer debugging experience:

```bash
# Run with TUI mode enabled
dbug run --tui /path/to/rust/project
```

In TUI mode, you have access to multiple panels showing different aspects of the debugging session:

1. **Source panel**: Shows the source code with the current execution point highlighted
2. **Variables panel**: Displays variables and their values in the current scope
3. **Call stack panel**: Shows the function call hierarchy
4. **Watches panel**: Displays watched expressions and their values
5. **Breakpoints panel**: Lists all breakpoints and their status
6. **Console panel**: Input commands and see their output

Keyboard shortcuts in TUI mode:

- `Tab`: Switch between panels
- `Up/Down`: Navigate within a panel
- `F5`: Continue execution
- `F10`: Step over
- `F11`: Step into
- `Shift+F11`: Step out
- `Ctrl+C` or `q`: Quit the debugger

## Variable Inspection

Dbug provides rich variable inspection capabilities:

```rust
use dbug::prelude::*;

#[dbug]
fn process_data(data: Vec<String>) {
    let count = data.len();
    dbug::break_here!(); // Inspect variables here
    
    // ...
}
```

When the breakpoint is hit, you can:

```
# Print a variable
p count

# Print a complex expression
p data.iter().filter(|s| s.contains("error")).count()

# Add a watch
w data.len()

# Register a variable for default visualization
dbug::register_var!(my_complex_struct);
```

## Common Troubleshooting Tips

### Breakpoints Not Triggering

If your breakpoints aren't triggering, check:

1. Are you running the build with `dbug build` or `dbug run`?
2. Is the `dbug` macro correctly applied to the function?
3. Have you imported `dbug::prelude::*`?

### Performance Issues

If you're experiencing performance issues:

1. Disable debug instrumentation for performance-critical functions
2. Use conditional breakpoints instead of breaking on every iteration
3. Limit the number of watched variables

### Communication Issues

If you see "Failed to communicate with debugger" errors:

1. Check if a previous debugging session didn't clean up properly 
2. Verify permissions for creating files in your system's temp directory
3. Try running with `dbug run --clean` to clear any stale debug files

## Requirements

- Rust 1.70.0 or higher
- Cargo

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request 