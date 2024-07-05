// create-empty-caption-files\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, write_to_file };
use std::path::{ Path, PathBuf };
use anyhow::Result;

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
    let directory = Path::new("E:\\training_dir_staging");
    walk_directory(directory, "jpg", |path| create_caption_file(path.to_path_buf())).await?;
    walk_directory(directory, "jpeg", |path| create_caption_file(path.to_path_buf())).await?;
    walk_directory(directory, "png", |path| create_caption_file(path.to_path_buf())).await?;
    println!("All caption files have been created.");
    Ok(())
}
