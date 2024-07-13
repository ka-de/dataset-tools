use std::path::Path;
use tokio::fs;
use walkdir::{ DirEntry, WalkDir };
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let target_dir = std::env::current_dir()?;
    remove_macos_artifacts(&target_dir).await?;
    println!("Finished removing macOS artifacts.");
    Ok(())
}

async fn remove_macos_artifacts(target_dir: &Path) -> Result<()> {
    let mut tasks = Vec::new();

    for entry in WalkDir::new(target_dir).follow_links(true).into_iter().filter_map(Result::ok) {
        if is_macos_artifact(&entry) {
            let path = entry.path().to_owned();
            let task = tokio::spawn(async move {
                if path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(&path).await {
                        eprintln!("Failed to remove directory {}: {}", path.display(), e);
                    } else {
                        println!("Removed directory: {}", path.display());
                    }
                } else {
                    if let Err(e) = fs::remove_file(&path).await {
                        eprintln!("Failed to remove file {}: {}", path.display(), e);
                    } else {
                        println!("Removed file: {}", path.display());
                    }
                }
            });
            tasks.push(task);
        }
    }

    for task in tasks {
        task.await?;
    }

    Ok(())
}

fn is_macos_artifact(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy();
    file_name == "__MACOSX" || file_name == ".DS_Store"
}
