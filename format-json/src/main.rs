// format-json\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

/// This script is used to format JSON files in a directory and its subdirectories.
/// It takes an optional command line argument which is the path to the directory.
/// If no argument is provided, it uses a default directory path.
/// It uses the `serde_json` crate to parse and format the JSON files,
/// and the `walkdir` crate to recursively traverse directories.

use dataset_tools::{ walk_directory, format_json_file };
use std::env;
use std::path::Path;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let directory_path = args.get(1).map_or("E:/projects/yiff_toolkit", String::as_str);

    walk_directory(Path::new(directory_path), "json", |path| {
        Box::pin(async move { format_json_file(path).await.map_err(Into::into) })
    }).await?;

    Ok(())
}
