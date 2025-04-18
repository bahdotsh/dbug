use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::io::{self, Write};
use clap::{Parser, Subcommand};

mod cli;
mod compiler;
mod runtime;
mod instrumentation;
mod utils;

#[derive(Parser)]
#[command(name = "dbug")]
#[command(about = "A CLI-based debugger for Rust projects", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a Rust project with debug instrumentation
    Build {
        /// Path to the Rust project
        #[arg(value_name = "PROJECT_PATH")]
        project_path: String,
        
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        
        /// Custom target directory
        #[arg(long, value_name = "DIR")]
        target_dir: Option<String>,
    },
    
    /// Build and run a Rust project with the debugger
    Run {
        /// Path to the Rust project
        #[arg(value_name = "PROJECT_PATH")]
        project_path: String,
        
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
    
    /// Build and debug a Rust project
    Debug {
        /// Path to the Rust project
        #[arg(value_name = "PROJECT_PATH")]
        project_path: String,
        
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Build { project_path, release, target_dir } => {
            build_project(project_path, *release, target_dir.as_deref());
        }
        Commands::Run { project_path, release } => {
            run_project(project_path, *release);
        }
        Commands::Debug { project_path, release } => {
            debug_project(project_path, *release);
        }
    }
}

fn build_project(project_path: &str, release: bool, target_dir: Option<&str>) {
    println!("Building project at: {}", project_path);
    
    // Check if the project is valid
    let cargo_toml_path = Path::new(project_path).join("Cargo.toml");
    if !cargo_toml_path.exists() {
        println!("Error: Invalid Rust project at '{}' (no Cargo.toml found)", project_path);
        exit(1);
    }
    
    // Find all Rust files in the project
    println!("Scanning project for Rust files...");
    let rust_files = match utils::find_rust_files(project_path) {
        Ok(files) => files,
        Err(e) => {
            println!("Error scanning project: {}", e);
            exit(1);
        }
    };
    
    println!("Found {} Rust files", rust_files.len());
    
    // Create the instrumenter
    let instrumenter = instrumentation::Instrumenter::new(project_path);
    
    // Find debug points in all files
    println!("Scanning for debug points...");
    let mut all_debug_points = 0;
    
    for file_path in &rust_files {
        let relative_path = match file_path.strip_prefix(Path::new(project_path)) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => file_path.to_string_lossy().to_string(),
        };
        
        let debug_points = instrumenter.find_debug_points(&relative_path);
        all_debug_points += debug_points.len();
        
        if !debug_points.is_empty() {
            match instrumenter.instrument_file(&relative_path, &debug_points) {
                Ok(_) => {},
                Err(e) => println!("Warning: Failed to instrument {}: {}", relative_path, e),
            }
        }
    }
    
    println!("Found {} debug points total", all_debug_points);
    
    // Build the project
    println!("Building project...");
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_path);
    cmd.arg("build");
    
    if release {
        cmd.arg("--release");
    }
    
    if let Some(dir) = target_dir {
        cmd.args(["--target-dir", dir]);
    }
    
    let status = cmd.status().expect("Failed to execute cargo build");
    
    if !status.success() {
        println!("Error: Build failed");
        exit(status.code().unwrap_or(1));
    }
    
    println!("Build successful! Project is ready for debugging.");
}

fn run_project(project_path: &str, release: bool) {
    println!("Building and running project at: {}", project_path);
    
    // Build first
    build_project(project_path, release, None);
    
    // For now, just pass through to cargo run
    println!("Starting debugger...");
    println!("Note: Full debugging capabilities will be implemented in future versions.");
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(project_path);
    cmd.arg("run");
    
    if release {
        cmd.arg("--release");
    }
    
    let status = cmd.status().expect("Failed to execute cargo run");
    
    if !status.success() {
        println!("Error: Run failed");
        exit(status.code().unwrap_or(1));
    }
}

fn debug_project(project_path: &str, release: bool) {
    println!("Debugging project at: {}", project_path);
    
    // Build first
    build_project(project_path, release, None);
    
    // Initialize the debugger
    println!("Starting debugger...");
    println!("Note: Full debugging capabilities will be implemented in future versions.");
    
    // Create and start the debugger CLI
    let mut debugger_cli = cli::DebuggerCli::new();
    debugger_cli.start();
    
    // Main debugging loop
    let mut command = String::new();
    loop {
        print!("dbug> ");
        io::stdout().flush().unwrap();
        
        command.clear();
        io::stdin().read_line(&mut command).unwrap();
        
        // Process the command
        debugger_cli.process_command(&command);
        
        // Check if we should exit
        match command.trim() {
            "quit" | "q" => break,
            _ => {} // Continue the loop
        }
    }
    
    println!("Debugging session ended.");
}
