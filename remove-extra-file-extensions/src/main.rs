// remove-extra-file-extensions\src\main.rs

// This program removes extra file extensions from .txt files in a target directory and subdirectories.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, rename_file_without_image_extension };
use std::env;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).map_or("E:/training_dir_staging", String::as_str);

    walk_directory(Path::new(dir), "txt", |path| {
        let path_buf = path.to_path_buf();
        async move {
            rename_file_without_image_extension(&path_buf).await.map_err(anyhow::Error::from)
        }
    }).await?;

    Ok(())
}
