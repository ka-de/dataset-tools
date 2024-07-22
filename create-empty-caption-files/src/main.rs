// create-empty-caption-files\src\main.rs

// This program creates empty caption files for all image files in a directory.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, write_to_file };
use std::path::{ Path, PathBuf };
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
    let directory = env
        ::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

    println!("Processing directory: {}", directory.display());

    walk_directory(&directory, "jpg", |path| create_caption_file(path.to_path_buf())).await?;
    walk_directory(&directory, "jpeg", |path| create_caption_file(path.to_path_buf())).await?;
    walk_directory(&directory, "png", |path| create_caption_file(path.to_path_buf())).await?;

    println!("All caption files have been created.");
    Ok(())
}
