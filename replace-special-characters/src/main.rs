// replace-special-characters\src\main.rs

// This program replaces some special characters LLMs like to spit out with versions that are
// actually on our keyboards in .txt files in a target directory and subdirectories.

use std::path::Path;
use anyhow::{ Result, Context };
use dataset_tools::{ walk_directory, read_file_content, write_to_file };
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the target directory from command line arguments or use the current directory
    let target_dir = env
        ::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());
    let target_path = Path::new(&target_dir);

    println!("Processing files in directory: {}", target_path.display());

    // Walk through the directory and process text files
    walk_directory(target_path, "txt", process_text_file).await?;

    println!("Processing complete.");
    Ok(())
}

async fn process_text_file(path: std::path::PathBuf) -> Result<()> {
    println!("Processing file: {}", path.display());

    // Read the content of the file
    let content = read_file_content(path.to_str().context("Invalid path")?).await?;

    // Replace the characters
    let processed_content = content.replace('’', "'").replace('“', "\"").replace('”', "\"");

    // Write the processed content back to the file
    write_to_file(&path, &processed_content).await?;

    Ok(())
}
