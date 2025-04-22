use clap::{Parser, Subcommand};
use dbug::{self};
use std::path::Path;
use std::process::{exit, Command};

// Temporarily commented out due to compilation issues
// mod cli;

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
        Commands::Build {
            project_path,
            release,
            target_dir,
        } => {
            build_project(project_path, *release, target_dir.as_deref());
        }
        Commands::Run {
            project_path,
            release,
        } => {
            run_project(project_path, *release);
        }
        Commands::Debug {
            project_path,
            release,
        } => {
            debug_project(project_path, *release);
        }
    }
}

fn build_project(project_path: &str, release: bool, _target_dir: Option<&str>) {
    println!("Building project at: {}", project_path);

    // Check if the project is valid
    if !dbug::cargo::is_cargo_project(project_path) {
        println!(
            "Error: Invalid Rust project at '{}' (no Cargo.toml found)",
            project_path
        );
        exit(1);
    }

    // Find all Rust files in the project
    println!("Scanning project for Rust files...");
    let rust_files = match dbug::utils::find_rust_files(project_path) {
        Ok(files) => files,
        Err(e) => {
            println!("Error scanning project: {}", e);
            exit(1);
        }
    };

    println!("Found {} Rust files", rust_files.len());

    // Create the instrumenter
    let instrumenter = dbug::instrumentation::Instrumenter::new(project_path);

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
                Ok(_) => {}
                Err(e) => println!("Warning: Failed to instrument {}: {}", relative_path, e),
            }
        }
    }

    println!("Found {} debug points total", all_debug_points);

    // Build the project with instrumentation
    println!("Building project...");

    match dbug::cargo::build_with_instrumentation(project_path, release) {
        Ok(_) => {
            println!("Build successful! Project is ready for debugging.");
        }
        Err(e) => {
            println!("Error building project: {}", e);
            exit(1);
        }
    }
}

fn run_project(project_path: &str, release: bool) {
    println!("Building and running project at: {}", project_path);

    // Build first
    build_project(project_path, release, None);

    // For now, just pass through to cargo run
    println!("Starting application with debug instrumentation...");

    // Initialize the communication channel
    if let Err(e) = dbug::communication::init_debugging_session() {
        println!("Error initializing debugging session: {}", e);
        exit(1);
    }

    // Find the executable path
    let project_name = match dbug::cargo::get_project_name(project_path) {
        Ok(name) => name,
        Err(e) => {
            println!("Error getting project name: {}", e);
            exit(1);
        }
    };

    // Use the utility function to find the correct executable path
    let executable_path = dbug::utils::find_executable_path(project_path, &project_name, release);

    // Run the executable directly with debugging enabled
    let status = match Command::new(&executable_path)
        .env("DBUG_ENABLED", "1")
        .status()
    {
        Ok(status) => status,
        Err(e) => {
            println!("Error launching executable: {}", e);
            exit(1);
        }
    };

    // Clean up
    if let Err(e) = dbug::communication::cleanup_debugging_session() {
        println!("Error cleaning up debugging session: {}", e);
    }

    if !status.success() {
        println!(
            "Error: Run failed with exit code: {}",
            status.code().unwrap_or(1)
        );
        exit(status.code().unwrap_or(1));
    }
}

fn debug_project(project_path: &str, release: bool) {
    println!("Debugging project at: {}", project_path);

    // Check if the project is valid
    if !dbug::cargo::is_cargo_project(project_path) {
        println!(
            "Error: Invalid Rust project at '{}' (no Cargo.toml found)",
            project_path
        );
        exit(1);
    }

    // Initialize a debugging session
    match dbug::session::get_current_session() {
        Ok(session) => {
            let mut session = session.lock().unwrap();
            if let Err(e) = session.start(project_path) {
                println!("Error starting debugging session: {}", e);
                exit(1);
            }
        }
        Err(e) => {
            println!("Error getting debugging session: {}", e);
            exit(1);
        }
    }

    // Build with instrumentation
    if let Err(e) = dbug::cargo::build_with_instrumentation(project_path, release) {
        println!("Error building project: {}", e);
        exit(1);
    }

    // Find the executable path
    let project_name = match dbug::cargo::get_project_name(project_path) {
        Ok(name) => name,
        Err(e) => {
            println!("Error getting project name: {}", e);
            exit(1);
        }
    };

    // Use the utility function to find the correct executable path
    let executable_path = dbug::utils::find_executable_path(project_path, &project_name, release);

    println!("Starting debugger for: {}", executable_path.display());

    // Initialize the communication channel
    if let Err(e) = dbug::communication::init_debugging_session() {
        println!("Error initializing debugging session: {}", e);
        exit(1);
    }

    // Set the executable in the session
    if let Ok(session) = dbug::session::get_current_session() {
        let mut session = session.lock().unwrap();
        if let Err(e) = session.set_executable(&executable_path) {
            println!("Warning: Could not set executable path: {}", e);
        }
    }

    // Launch the executable in a separate process
    let child_process = match std::process::Command::new(&executable_path)
        .env("DBUG_ENABLED", "1") // Signal to the program that it's being debugged
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            println!("Error launching executable: {}", e);
            // Clean up
            dbug::communication::cleanup_debugging_session().unwrap_or_else(|e| {
                println!("Error cleaning up debugging session: {}", e);
            });
            exit(1);
        }
    };

    // Store the child process ID in the session
    let pid = child_process.id();
    println!("Debugging process with PID: {}", pid);

    if let Ok(session) = dbug::session::get_current_session() {
        let mut session = session.lock().unwrap();
        if let Err(e) = session.set_debugged_pid(pid) {
            println!("Warning: Could not set debugged PID: {}", e);
        }
    }

    // Launch the TUI
    match dbug::tui::run() {
        Ok(_) => println!("TUI session completed"),
        Err(e) => println!("Error running TUI: {}", e),
    }

    // Terminate the child process
    if let Ok(session) = dbug::session::get_current_session() {
        if let Err(e) = session.lock().unwrap().stop() {
            println!("Warning: Could not properly stop debugging session: {}", e);
        }
    }

    // Clean up
    if let Err(e) = dbug::communication::cleanup_debugging_session() {
        println!("Error cleaning up debugging session: {}", e);
    }

    println!("Debugging session ended.");
}
