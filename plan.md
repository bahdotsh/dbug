# Dbug: A CLI-based Debugger for Rust Projects

## Overview

Dbug is a CLI-based debugger for Rust projects that aims to simplify the debugging experience. It allows users to place debug points in their code and provides an intuitive interface for debugging Rust applications without the complexity of traditional debuggers.

## Core Features

1. **Simple debug point insertion** - Allow users to add debug points with minimal syntax
2. **Compile-time integration** - Act as a wrapper around the Rust compiler
3. **Runtime value inspection** - View variables, types, and memory at debug points
4. **Flow control** - Step through code execution, continue, or break
5. **Conditional breakpoints** - Break only when specific conditions are met
6. **Watch expressions** - Monitor variables or expressions as they change
7. **Easy-to-use CLI interface** - Clear, color-coded output and simple commands

## Implementation Plan

### 1. Architecture

The debugger will consist of three main components:

1. **CLI Frontend**: Handles user interactions and commands
2. **Compiler Integration**: Processes source code and integrates with Rust's build system
3. **Runtime Engine**: Manages program execution and breakpoint handling

### 2. Technical Approach

#### 2.1 Debug Points Implementation

We'll use a procedural macro approach to transform debug points in the user's code. Users will be able to add debug points using a simple macro:

```rust
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

#### 2.2 Compiler Integration

Dbug will wrap the Rust compiler and:

1. Parse the source code to locate debug annotations
2. Insert instrumentation code at debug points
3. Generate debug information
4. Compile the project with appropriate flags

We'll use the `--emit=mir` and other compiler flags to gather necessary information and hook into the compilation process.

#### 2.3 Runtime Engine

The runtime engine will:

1. Load the compiled program
2. Set up handlers for debug points
3. Manage program execution (stepping, continuing, etc.)
4. Gather and display variable information at breakpoints

We'll use a combination of LLDB/GDB behind the scenes but abstract away their complexity.

### 3. Development Phases

#### Phase 1: Core Infrastructure

1. Create basic CLI structure with Clap or similar
2. Implement compiler wrapper that passes through to rustc
3. Develop source code parsing for debug point detection
4. Build initial instrumentation system for simple breakpoints

#### Phase 2: Basic Debugging Features

1. Implement breakpoint mechanism
2. Add support for inspecting variables at breakpoints
3. Develop basic flow control (continue, step, next)
4. Create simple CLI interface for debug session

#### Phase 3: Advanced Features

1. Add conditional breakpoints
2. Implement watch expressions
3. Support for complex data structure visualization
4. Add support for remote debugging

#### Phase 4: Polish and Performance

1. Optimize performance for large projects
2. Improve error handling and user feedback
3. Enhance documentation and examples
4. Add integration with common editors/IDEs

### 4. Technical Challenges

1. **Rust Compiler Integration**: Working with the Rust compiler internals can be complex
2. **Performance Overhead**: Ensuring minimal performance impact when debugging
3. **Cross-Platform Support**: Working consistently across different operating systems
4. **Handling Optimizations**: Dealing with compiler optimizations that might affect debugging

### 5. Implementation Details

#### 5.1 Source Transformation

We'll use the `syn` and `quote` crates to parse and transform Rust source code. When a debug point is encountered, we'll insert code that communicates with the debugger runtime.

#### 5.2 Debug Information Management

We'll generate and manage debug information using:

1. DWARF debug information from rustc
2. Custom metadata for our debug points

#### 5.3 Runtime Communication

The inserted instrumentation will communicate with the debugger runtime through a combination of:

1. Shared memory for efficiency
2. Pipes or sockets for command transfer
3. Signal handling for breakpoint notification

#### 5.4 User Interface

The CLI interface will provide:

1. Color-coded source code display
2. Variable inspection with syntax highlighting
3. Simple command syntax for common operations
4. Help system and contextual suggestions

### 6. Technology Stack

1. **Rust**: The entire debugger will be written in Rust
2. **syn/quote/proc-macro2**: For code parsing and transformation
3. **LLDB/GDB**: As the underlying debugging engine
4. **Clap**: For CLI argument parsing
5. **crossterm/tui-rs**: For terminal UI components

### 7. Project Structure

```
dbug/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # CLI interface module
│   ├── compiler/            # Compiler integration module
│   ├── runtime/             # Debug runtime module
│   ├── instrumentation/     # Code instrumentation module
│   └── utils/               # Helper utilities
├── proc_macros/             # Procedural macros for debug annotations
├── examples/                # Example Rust projects to debug
└── tests/                   # Integration tests
```

### 8. Timeline

- **Month 1**: Core infrastructure and basic compiler integration
- **Month 2**: Basic debugging features and initial CLI
- **Month 3**: Advanced features and optimization
- **Month 4**: Testing, documentation, and first release

## Conclusion

The Dbug project aims to create a user-friendly debugging experience for Rust developers by simplifying the process of setting breakpoints and inspecting program state. By leveraging Rust's powerful macro system and compiler capabilities, we can create a seamless debugging experience without requiring complex external tools or configurations. 

## Implementation Progress

### Core Features Implementation Status

- [x] Project scaffolding and basic structure
- [x] Basic command-line interface
- [x] Debug point insertion syntax
  - [x] Basic syntax definitions
  - [x] Full implementation of debug point detection
- [x] Simple compiler wrapper
- [x] Runtime value inspection
- [x] Flow control
- [x] Conditional breakpoints
- [x] Watch expressions

### Phase 1: Core Infrastructure

- [x] Create project structure with appropriate modules
- [x] Basic CLI structure
  - [x] Command parsing
  - [x] Help and version commands 
  - [x] Integration with Clap (upgraded from manual parsing)
- [x] Implement compiler wrapper that passes through to rustc
  - [x] Basic cargo build pass-through
  - [x] Advanced compiler flag management
- [x] Source code parsing for debug point detection
  - [x] Module structure
  - [x] Implemented enhanced debug point detection with both text-based and AST-based parsing
- [x] Basic instrumentation system for breakpoints
  - [x] Interface definition
  - [x] Working macros for function instrumentation and breakpoints
  - [x] Basic implementation with output for debugging

### Phase 2: Basic Debugging Features

- [x] Basic breakpoint mechanism
  - [x] Data structures
  - [x] Basic runtime printing (prints breakpoint location)
  - [x] Interactive breakpoint handling
- [x] Variable inspection
  - [x] Support for various variable types
  - [x] Variable value representation 
  - [x] Scope management
- [x] Flow control (step, next, continue)
  - [x] Step over implementation
  - [x] Step into implementation
  - [x] Continue execution
- [x] Basic CLI interface for debug session
  - [x] Command loop
  - [x] Help information
  - [x] Interactive debugging
  - [x] Source code display with current line highlighting

### Phase 3: Advanced Features

- [x] Conditional breakpoints
- [x] Watch expressions
- [x] Complex data structure visualization
- [ ] Remote debugging

### Phase 4: Polish and Performance

- [x] Performance optimization
- [x] Comprehensive error handling
- [x] Basic documentation
  - [x] README
  - [x] Implementation plan
  - [ ] API documentation
- [ ] Editor/IDE integration
- [x] Working integration tests for basic functionality 