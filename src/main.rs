use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "build" => {
            if args.len() < 3 {
                println!("Error: Please provide a path to the Rust project");
                exit(1);
            }
            let project_path = &args[2];
            build_project(project_path);
        }
        "run" => {
            if args.len() < 3 {
                println!("Error: Please provide a path to the Rust project");
                exit(1);
            }
            let project_path = &args[2];
            run_project(project_path);
        }
        "debug" => {
            if args.len() < 3 {
                println!("Error: Please provide a path to the Rust project");
                exit(1);
            }
            let project_path = &args[2];
            debug_project(project_path);
        }
        "help" => {
            print_usage();
        }
        "version" => {
            println!("dbug v0.1.0");
        }
        _ => {
            println!("Error: Unknown command '{}'", command);
            print_usage();
            exit(1);
        }
    }
}

fn print_usage() {
    println!("dbug - A CLI-based debugger for Rust projects");
    println!("");
    println!("USAGE:");
    println!("  dbug [COMMAND] [OPTIONS]");
    println!("");
    println!("COMMANDS:");
    println!("  build <project_path>    Build a Rust project with debug instrumentation");
    println!("  run <project_path>      Build and run a Rust project with the debugger");
    println!("  debug <project_path>    Build and debug a Rust project");
    println!("  help                    Print this help message");
    println!("  version                 Print version information");
}

fn build_project(project_path: &str) {
    println!("Building project at: {}", project_path);
    
    // Check if the project is valid
    let cargo_toml_path = std::path::Path::new(project_path).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        println!("Error: Invalid Rust project at '{}' (no Cargo.toml found)", project_path);
        exit(1);
    }
    
    // Find all Rust files in the project
    println!("Scanning project for Rust files...");
    println!("Scanning for debug points...");
    println!("Note: Full debug point detection will be implemented in future versions.");
    
    // Build the project
    println!("Building project...");
    let status = Command::new("cargo")
        .current_dir(project_path)
        .args(&["build"])
        .status()
        .expect("Failed to execute cargo build");
        
    if !status.success() {
        println!("Error: Build failed");
        exit(status.code().unwrap_or(1));
    }
    
    println!("Build successful! Project is ready for debugging.");
}

fn run_project(project_path: &str) {
    println!("Building and running project at: {}", project_path);
    
    // Build first
    build_project(project_path);
    
    // For now, just pass through to cargo run
    println!("Starting debugger...");
    println!("Note: Full debugging capabilities will be implemented in future versions.");
    
    let status = Command::new("cargo")
        .current_dir(project_path)
        .args(&["run"])
        .status()
        .expect("Failed to execute cargo run");
        
    if !status.success() {
        println!("Error: Run failed");
        exit(status.code().unwrap_or(1));
    }
}

fn debug_project(project_path: &str) {
    println!("Debugging project at: {}", project_path);
    
    // Build first
    build_project(project_path);
    
    // Initialize the debugger
    println!("Starting debugger...");
    println!("Note: Full debugging capabilities will be implemented in future versions.");
    
    // Main debugging loop
    let mut command = String::new();
    loop {
        print!("dbug> ");
        io::stdout().flush().unwrap();
        
        command.clear();
        io::stdin().read_line(&mut command).unwrap();
        
        // Process the command
        match command.trim() {
            "help" => print_debug_help(),
            "quit" | "q" => {
                println!("Exiting debugger");
                break;
            }
            _ => println!("Unknown command: {}. Type 'help' for a list of commands.", command.trim()),
        }
    }
    
    println!("Debugging session ended.");
}

fn print_debug_help() {
    println!("Dbug Debugger Commands:");
    println!("  n, next            Step to the next line");
    println!("  s, step            Step into a function call");
    println!("  c, continue        Continue execution until the next breakpoint");
    println!("  p, print <expr>    Print the value of an expression");
    println!("  w, watch <expr>    Watch an expression for changes");
    println!("  q, quit            Quit the debugger");
    println!("  help               Show this help message");
}
