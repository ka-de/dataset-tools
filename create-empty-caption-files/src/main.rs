// create-empty-caption-files\src\main.rs

// This program creates empty caption files for all image files in a directory.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, write_to_file };
use std::path::PathBuf;
use anyhow::Result;
use std::env;

async fn create_caption_file(path: PathBuf) -> Result<()> {
    let caption_file = path.with_extension("txt");
    if !caption_file.exists() {
        write_to_file(&caption_file, "").await?;
        println!("Created caption file: {}", caption_file.display());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let directory = args.get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

    let extension = args.get(2).unwrap_or(&"txt".to_string()).clone();

    println!("Processing directory: {}", directory.display());

    // Use the specified extension when creating caption files
    walk_directory(&directory, "webp", |path| create_caption_file(path.with_extension(&extension))).await?;
    walk_directory(&directory, "jxl", |path| create_caption_file(path.with_extension(&extension))).await?;
    walk_directory(&directory, "jpg", |path| create_caption_file(path.with_extension(&extension))).await?;
    walk_directory(&directory, "jpeg", |path| create_caption_file(path.with_extension(&extension))).await?;
    walk_directory(&directory, "png", |path| create_caption_file(path.with_extension(&extension))).await?;

    println!("All caption files have been created.");
    Ok(())
}
