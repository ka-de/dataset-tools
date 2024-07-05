// convert-caption-json-to-txt/src/main.rs
//
// Converts the json created by JTP_PILOT2-2-e3-vit_so400m_patch14_siglip_384
// to caption files.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::path::Path;
use walkdir::WalkDir;
use dataset_tools::process_json_to_caption;
use tokio::fs::File;
use tokio::io::{ AsyncBufReadExt, BufReader };

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_directory_or_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    if input_path.is_dir() {
        let mut tasks = Vec::new();
        for entry in WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path().to_owned();
            if path.is_file() {
                tasks.push(
                    tokio::spawn(async move {
                        if let Err(e) = process_json_to_caption(&path).await {
                            eprintln!("Error processing {}: {}", path.display(), e);
                        }
                    })
                );
            }
        }
        for task in tasks {
            task.await?;
        }
    } else if input_path.is_file() {
        let file = File::open(input_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            let path = Path::new(&line);
            if path.exists() {
                if let Err(e) = process_json_to_caption(path).await {
                    eprintln!("Error processing {}: {}", path.display(), e);
                }
            } else {
                eprintln!("File not found: {line}");
            }
        }
    } else {
        eprintln!("Invalid input: not a directory or file");
        std::process::exit(1);
    }

    Ok(())
}
