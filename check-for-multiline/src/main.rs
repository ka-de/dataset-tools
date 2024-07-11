use anyhow::{ Context, Result };
use dataset_tools::{ walk_directory, read_file_content };
use std::env;
use std::path::PathBuf;
use tokio::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::process;

#[tokio::main]
async fn main() {
    // Get the directory path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory_path>", args[0]);
        process::exit(1);
    }
    let directory_path = &args[1];

    // Vector to store paths of files with multiple lines
    let multi_line_files = Arc::new(Mutex::new(Vec::new()));

    // Walk through the directory and process txt files
    if
        let Err(e) = walk_directory(directory_path, "txt", |path| {
            let multi_line_files = Arc::clone(&multi_line_files);
            async move { process_file(path, multi_line_files).await }
        }).await
    {
        eprintln!("Error: {e}");
        process::exit(1);
    }

    // If there are files with multiple lines, open them in Neovim
    let files = multi_line_files.lock().await;
    if !files.is_empty() {
        println!("\nOpening files with multiple lines in Neovim...");
        if let Err(e) = open_files_in_neovim(&files).await {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    } else {
        println!("No files with multiple lines found.");
    }
}

async fn process_file(path: PathBuf, multi_line_files: Arc<Mutex<Vec<PathBuf>>>) -> Result<()> {
    let content = read_file_content(path.to_str().context("Invalid path")?).await?;
    let line_count = content.lines().count();

    if line_count > 1 {
        println!("File with multiple lines found: {}", path.display());
        multi_line_files.lock().await.push(path);
    }

    Ok(())
}

async fn open_files_in_neovim(files: &[PathBuf]) -> Result<()> {
    let file_paths: Vec<&str> = files
        .iter()
        .filter_map(|p| p.to_str())
        .collect();

    Command::new("nvim")
        .args(&file_paths)
        .spawn()
        .context("Failed to spawn Neovim")?
        .wait().await
        .context("Failed to wait for Neovim")?;

    Ok(())
}
