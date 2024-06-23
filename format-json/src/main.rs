// This script is used to format JSON files in a directory and its subdirectories.
// It takes an optional command line argument which is the path to the directory.
// If no argument is provided, it uses a default directory path.
// It uses the `serde_json` crate to parse and format the JSON files,
// and the `walkdir` crate to recursively traverse directories.

use serde_json::Value;
use std::fs;
use std::path::Path;
use std::io::Write;
use walkdir::WalkDir;
use std::env;

// The main function starts the script.
// It reads the command line arguments and sets the directory path.
// Then it traverses the directory and its subdirectories using WalkDir.
// For each JSON file it finds, it calls `format_json_file` to format it.
fn main() {
    // Read command line arguments
    let args: Vec<String> = env::args().collect();
    // Set directory path
    let directory_path = if args.len() > 1 {
        &args[1]
    } else {
        "E:/projects/yiff_toolkit/ponyxl_loras"
    };

    // Traverse directory and its subdirectories
    for entry in WalkDir::new(directory_path) {
        let entry = entry.unwrap();
        let path = entry.path();
        // If the file is a JSON file, format it
        if path.extension().unwrap_or_default() == "json" {
            if let Err(e) = format_json_file(path) {
                println!("Failed to format {}: {}", path.display(), e);
            }
        }
    }
}

// The `format_json_file` function formats a JSON file.
// It reads the file, parses the JSON, formats it, and writes it back to the file.
// If any of these steps fail, it returns an error.
fn format_json_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing file: {}", path.display());

    // Read the file
    let file_content = fs::read_to_string(path)?;
    // Parse the JSON
    let json: Value = serde_json::from_str(&file_content)?;
    // Format the JSON
    let pretty_json = serde_json::to_string_pretty(&json)?;
    // Write the formatted JSON back to the file
    let mut file = fs::File::create(path)?;
    file.write_all(pretty_json.as_bytes())?;

    println!("Formatted {} successfully.", path.display());
    Ok(())
}
