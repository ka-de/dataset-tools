// extract-metadata\src\main.rs

// This program extracts metadata from .safetensors files in a target directory and subdirectories.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, process_safetensors_file };
use std::env;
use std::path::Path;
use glob::glob;
use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger to output diagnostic information.
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename or directory>", args[0]);
        return Ok(());
    }
    let path = Path::new(&args[1]);

    if path.is_dir() {
        walk_directory(path, "safetensors", |file_path| {
            async move { process_safetensors_file(&file_path).await }
        }).await?;
    } else if let Some(path_str) = path.to_str() {
        if path_str.contains('*') {
            for entry in glob(path_str).context("Failed to read glob pattern")? {
                match entry {
                    Ok(path) => {
                        process_safetensors_file(&path).await?;
                    }
                    Err(e) => println!("Error processing entry: {:?}", e),
                }
            }
        } else {
            process_safetensors_file(path).await?;
        }
    } else {
        return Err(anyhow::anyhow!("Invalid path provided"));
    }

    Ok(())
}
