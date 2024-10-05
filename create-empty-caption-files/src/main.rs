// create-empty-caption-files\src\main.rs

// This program creates empty caption files for all image files in a directory.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, write_to_file };
use std::path::PathBuf;
use anyhow::Result;
use std::env;
use std::sync::Arc;

async fn create_caption_file(path: PathBuf, extension: &str) -> Result<()> {
    let caption_file = path.with_extension(extension);
    if !caption_file.exists() {
        write_to_file(&caption_file, "").await?;
        println!("Created caption file: {}", caption_file.display());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut directory = PathBuf::new();
    let mut extension = String::from("txt");

    for (i, arg) in args.iter().enumerate() {
        if arg == "--ext" && i + 1 < args.len() {
            extension = args[i + 1].clone();
        } else if i > 0 && !arg.starts_with("--") {
            directory = PathBuf::from(arg);
        }
    }

    if directory.as_os_str().is_empty() {
        directory = env::current_dir().expect("Failed to get current directory");
    }

    println!("Processing directory: {}", directory.display());
    println!("Using extension: .{}", extension);

    let image_extensions = ["webp", "jxl", "jpg", "jpeg", "png"];
    let extension = Arc::new(extension);

    for &ext in &image_extensions {
        let extension_clone = Arc::clone(&extension);
        walk_directory(&directory, ext, move |path| {
            let extension = extension_clone.clone();
            async move {
                create_caption_file(path.to_path_buf(), &extension).await
            }
        }).await?;
    }

    println!("All caption files have been created.");
    Ok(())
}
