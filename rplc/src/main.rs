// rplc\src\main.rs

// This program replaces a string in all .txt files in a target directory and subdirectories.
// It can also replace some special characters with their keyboard-friendly versions.
//
// Usage:
// - String replacement: ./rplc.exe <search_string> <replace_string> [target_dir]
// - Special character replacement: ./rplc.exe --apostrophes [target_dir]
//
// Examples:
// ./rplc.exe "foo" "bar" ./my_directory
// ./rplc.exe --apostrophes ./my_directory

#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::path::{ PathBuf, Path };
use anyhow::{ Result, Context };
use dataset_tools::{ walk_directory, format_text_content };
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  String replacement: ./rplc.exe <search_string> <replace_string> [target_dir]");
        eprintln!("  Special character replacement: ./rplc.exe --apostrophes [target_dir]");
        std::process::exit(1);
    }

    let target_dir = args
        .last()
        .map(PathBuf::from)
        .unwrap_or_else(|| ".".into());

    if args[1] == "--apostrophes" {
        println!("Replacing special characters in all .txt files in {:?}...", target_dir);
        walk_directory(&target_dir, "txt", replace_special_chars).await?;
    } else if args.len() >= 3 {
        let search_string = args[1].clone();
        let replace_string = args[2].clone();
        println!(
            "Replacing '{search_string}' with '{replace_string}' in all .txt files in {:?}...",
            target_dir
        );
        walk_directory(&target_dir, "txt", move |path| {
            let search = search_string.clone();
            let replace = replace_string.clone();
            async move { process_file(&path, &search, &replace).await }
        }).await?;
    } else {
        eprintln!("Invalid arguments. Use --help for usage information.");
        std::process::exit(1);
    }

    println!("Processing complete.");
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

async fn replace_special_chars(path: PathBuf) -> Result<()> {
    let content = fs::read_to_string(&path).await.context("Failed to read file")?;
    let new_content = content.replace('\'', "'").replace('"', "\"").replace('"', "\"");

    if content != new_content {
        fs::write(&path, new_content).await.context("Failed to write file")?;
        println!("Updated: {}", path.display());
    }

    Ok(())
}
