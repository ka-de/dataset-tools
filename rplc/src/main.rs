// rplc\src\main.rs

// This program replaces a string in all .txt files in a target directory and subdirectories.
//
// Usage: ./rplc.exe <search_string> <replace_string>
// Example: ./rplc.exe "foo" "bar"

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::path::Path;
use anyhow::Result;
use dataset_tools::{ walk_directory, format_text_content };
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: ./rplc.exe <search_string> <replace_string>");
        std::process::exit(1);
    }

    let search_string = args[1].clone();
    let replace_string = args[2].clone();

    println!("Replacing '{search_string}' with '{replace_string}' in all .txt files...");

    walk_directory(".", "txt", move |path| {
        let search = search_string.clone();
        let replace = replace_string.clone();
        async move { process_file(&path, &search, &replace).await }
    }).await?;

    println!("Replacement and formatting complete.");
    Ok(())
}

async fn process_file(path: &Path, search: &str, replace: &str) -> Result<()> {
    let content = fs::read_to_string(path).await?;
    let mut new_content = content.replace(search, replace);

    if replace.is_empty() {
        new_content = format_text_content(&new_content)?;
    }

    if content != new_content {
        fs::write(path, new_content).await?;
        println!("Updated: {}", path.display());
    }

    Ok(())
}
