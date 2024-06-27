// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, process_safetensors_file };
use std::env;
use std::path::Path;
use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename or directory>", args[0]);
        return Ok(());
    }
    let path = Path::new(&args[1]);

    if path.is_dir() {
        walk_directory(path, "safetensors", |file_path| {
            if let Err(err) = process_safetensors_file(file_path) {
                eprintln!("Error processing {file_path:#?}: {err}");
            }
            Ok(())
        })?;
    } else {
        process_safetensors_file(path)?;
    }
    Ok(())
}
