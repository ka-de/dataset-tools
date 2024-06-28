// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::{ PathBuf, Path };
use dataset_tools::{ walk_rust_files, read_lines, write_to_file };
use anyhow::{ Result, Context, anyhow };

const WARNING_COMMENT: &str =
    r"// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]
";

fn insert_warning(path: &Path) -> Result<()> {
    // Check if the file has a .rs extension
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Err(anyhow!("File is not a Rust source file: {}", path.display()));
    }

    let lines = read_lines(path).context("Failed to read file")?;

    // Check if the warning is already present in the first two lines
    if
        lines.len() >= 2 &&
        (lines[0].contains("#![warn(clippy::all, clippy::pedantic)]") ||
            lines[1].contains("#![warn(clippy::all, clippy::pedantic)]"))
    {
        println!("Warning already present in: {}", path.display());
        return Ok(());
    }

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

fn process_files(target: &Path) -> Result<()> {
    if target.is_file() {
        match insert_warning(target) {
            Ok(()) => println!("Processed file: {}", target.display()),
            Err(e) => println!("Error processing file {}: {}", target.display(), e),
        }
    } else if target.is_dir() {
        walk_rust_files(target, |path| {
            match insert_warning(path) {
                Ok(()) => {
                    println!("Processed file: {}", path.display());
                    Ok(())
                }
                Err(e) => {
                    println!("Error processing file {}: {}", path.display(), e);
                    Ok(())
                }
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
