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
- `c` or `continue`: Continue execution until the next breakpoint
- `p <expression>` or `print <expression>`: Print the value of an expression
- `w <expression>` or `watch <expression>`: Watch an expression for changes
- `q` or `quit`: Quit the debugger

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