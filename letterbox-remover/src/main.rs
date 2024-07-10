// letterbox-remover/src/main.rs

// This program removes letterboxes from images in a target directory and subdirectories.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::path::Path;
use walkdir::WalkDir;
use dataset_tools::remove_letterbox;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_directory>", args[0]);
        std::process::exit(1);
    }

    let input_dir = Path::new(&args[1]);

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && dataset_tools::is_image_file(path) {
            println!("Processing: {}", path.display());
            remove_letterbox(path)?;
        }
    }

    Ok(())
}
