// convert-caption-json-to-txt/src/main.rs
//
// Converts the json created by JTP_PILOT2-2-e3-vit_so400m_patch14_siglip_384
// to caption files.

use std::env;
use std::fs::File;
use std::io::{ BufRead, BufReader };
use std::path::Path;
use walkdir::WalkDir;
use dataset_tools::process_json_to_caption;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_directory_or_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    if input_path.is_dir() {
        for entry in WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                process_json_to_caption(path)?;
            }
        }
    } else if input_path.is_file() {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let path = Path::new(&line);
            if path.exists() {
                process_json_to_caption(path)?;
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
