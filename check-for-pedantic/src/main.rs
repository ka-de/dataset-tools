// check-for-pedantic\src\main.rs

// This program searches for pedantic warnings in Rust source files and reports the files
// that do not have them.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_rust_files, process_rust_file };
use anyhow::{ Result, Context };
use log::warn;
use std::{ path::PathBuf, process, env, sync::Arc };
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger to output diagnostic information.
    env_logger::init();

    // Collect command-line arguments.
    let args: Vec<String> = env::args().collect();
    // Get the current working directory as the default directory to check.
    let default_dir = env::current_dir()?;
    // Use the second command-line argument as the target directory, or default if not provided.
    let target = args.get(1).map_or(default_dir, PathBuf::from);

    // Resolve any relative path elements to get an absolute path.
    let canonical_target = target.canonicalize().context("Failed to canonicalize path")?;

    // Create a shared list to track files without the required warning.
    let files_without_warning = Arc::new(Mutex::new(Vec::<PathBuf>::new()));

    // Check if the target is a single file and if it's a Rust file.
    if canonical_target.is_file() && canonical_target.extension().map_or(false, |ext| ext == "rs") {
        // Clone the shared list for use in this scope.
        let files_without_warning_clone = Arc::clone(&files_without_warning);
        // Lock the list to prevent other operations from modifying it concurrently.
        let mut guard = files_without_warning_clone.lock().await;
        // Process the single Rust file.
        process_rust_file(&canonical_target, &mut *guard).await?;
        // Check if the target is a directory.
    } else if canonical_target.is_dir() {
        // Walk through all Rust files in the directory.
        walk_rust_files(&canonical_target, |path| {
            // Clone the shared list for use in this async block.
            let files_without_warning_clone = Arc::clone(&files_without_warning);
            // Convert the path reference to an owned PathBuf.
            let path_buf = path.to_path_buf();
            // Define an async block to process each Rust file.
            async move {
                // Lock the list to prevent other operations from modifying it concurrently.
                let mut guard = files_without_warning_clone.lock().await;
                // Process the Rust file.
                process_rust_file(&path_buf, &mut *guard).await
            }
        }).await.context("Failed to walk through Rust files")?;
        // If the target is neither a file nor a directory, log a warning and exit.
    } else {
        warn!("Invalid target. Please provide a .rs file or a directory.");
        process::exit(1);
    }

    // Lock the shared list to access its contents.
    let files_without_warning = files_without_warning.lock().await;
    // If there are any files without the required warning, print them out and exit with an error.
    if !files_without_warning.is_empty() {
        println!("The following files are missing the required warning:");
        for file in files_without_warning.iter() {
            println!("{}", file.display());
        }
        process::exit(1);
    }

    // If everything went well, return Ok.
    Ok(())
}
