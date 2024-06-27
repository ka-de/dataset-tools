// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::PathBuf;
use std::io;
use dataset_tools::{ walk_rust_files, read_lines, write_to_file };
use anyhow::{ Result, Context };

const WARNING_COMMENT: &str =
    r"// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]
";

fn insert_warning(path: &std::path::Path) -> Result<()> {
    let lines = read_lines(path).context("Failed to read file")?;
    let mut new_content = String::new();

    // Insert the warning comment at the beginning
    new_content.push_str(WARNING_COMMENT);
    new_content.push('\n');

    // Add the rest of the file content
    for line in lines {
        new_content.push_str(&line);
        new_content.push('\n');
    }

    write_to_file(path, &new_content).context("Failed to write to file")
}

fn process_files(target: &std::path::Path) -> Result<()> {
    if target.is_file() {
        insert_warning(target)?;
        println!("Inserted warning in: {}", target.display());
    } else if target.is_dir() {
        walk_rust_files(target.to_str().unwrap(), |path, _, _| {
            match insert_warning(path) {
                Ok(()) => {
                    println!("Inserted warning in: {}", path.display());
                    Ok(())
                }
                Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
            }
        })?;
    } else {
        println!("Invalid path: {}", target.display());
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <target_file_or_directory>", args[0]);
        std::process::exit(1);
    }

    let target = PathBuf::from(&args[1]);
    process_files(&target)?;

    Ok(())
}
