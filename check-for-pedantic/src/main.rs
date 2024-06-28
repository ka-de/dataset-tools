// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_rust_files, process_rust_file };
use anyhow::{ Result, Context };
use log::warn;
use env_logger;
use std::{ path::PathBuf, process, env };

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let default_dir = env::current_dir()?;
    let target = args.get(1).map(PathBuf::from).unwrap_or(default_dir);

    // Canonicalize the path to resolve `.` and `..`
    let canonical_target = target.canonicalize().context("Failed to canonicalize path")?;

    let mut files_without_warning = Vec::new();

    if canonical_target.is_file() && canonical_target.extension().map_or(false, |ext| ext == "rs") {
        process_rust_file(&canonical_target, &mut files_without_warning)?;
    } else if canonical_target.is_dir() {
        walk_rust_files(&canonical_target, |path| {
            process_rust_file(path, &mut files_without_warning)
        }).context("Failed to walk through Rust files")?;
    } else {
        warn!("Invalid target. Please provide a .rs file or a directory.");
        process::exit(1);
    }

    if !files_without_warning.is_empty() {
        println!("The following files are missing the required warning:");
        for file in &files_without_warning {
            println!("{}", file.display());
        }
        process::exit(1);
    }

    Ok(())
}
