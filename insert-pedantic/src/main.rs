// insert-pedantic\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::{ PathBuf, Path };
use dataset_tools::{ process_rust_file, walk_rust_files };
use anyhow::{ Result, Context, anyhow };
use tokio::{ fs, io };

const WARNING_COMMENT: &str =
    r"// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]
";

async fn insert_warning(path: &Path, files_without_warning: &mut Vec<PathBuf>) -> Result<()> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Err(anyhow!("File is not a Rust source file: {}", path.display()));
    }
    process_rust_file(path, files_without_warning).await.map_err(|e|
        io::Error::new(io::ErrorKind::Other, e)
    )?;
    if files_without_warning.contains(&path.to_path_buf()) {
        let content = fs::read_to_string(path).await.context("Failed to read file")?;
        let new_content = format!("{WARNING_COMMENT}{content}");
        fs::write(path, new_content).await.context("Failed to write to file")?;
        println!("Inserted warning in: {}", path.display());
    } else {
        println!("Warning already present in: {}", path.display());
    }
    Ok(())
}

async fn process_files(target: &Path) -> io::Result<()> {
    let mut files_without_warning = Vec::new();
    if target.is_file() {
        insert_warning(target, &mut files_without_warning).await.map_err(|e|
            io::Error::new(io::ErrorKind::Other, e)
        )?;
        println!("Processed file: {}", target.display());
    } else if target.is_dir() {
        walk_rust_files(target, |path| {
            {
                let mut value = files_without_warning.clone();
                async move {
                    insert_warning(&path, &mut value).await.map_err(|e|
                        io::Error::new(io::ErrorKind::Other, e)
                    )
                }
            }
        }).await?;
    } else {
        println!("Invalid path: {}", target.display());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <target_file_or_directory>", args[0]);
        return Ok(());
    }

    let target = PathBuf::from(&args[1]);
    process_files(&target).await?;

    Ok(())
}
