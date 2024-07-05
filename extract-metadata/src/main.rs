// extract-metadata\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, process_safetensors_file };
use std::env;
use std::path::Path;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger to output diagnostic information.
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename or directory>", args[0]);
        return Ok(());
    }
    let path = Path::new(&args[1]);

    Ok(
        (if path.is_dir() {
            walk_directory(path, "safetensors", |file_path| {
                async move {
                    process_safetensors_file(&file_path).await.map_err(|err|
                        std::io::Error::new(std::io::ErrorKind::Other, err)
                    )
                }
            }).await
        } else {
            process_safetensors_file(path).await.map_err(|err|
                std::io::Error::new(std::io::ErrorKind::Other, err)
            )
        })?
    )
}
