use dataset_tools::{ walk_directory, check_file_for_multiple_lines, open_files_in_neovim };
use std::env;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;

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
            async move {
                if !path.to_str().unwrap_or("").ends_with("-sample-prompts.txt") {
                    check_file_for_multiple_lines(path, multi_line_files).await
                } else {
                    Ok(())
                }
            }
        }).await
    {
        eprintln!("Error: {}", e);
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
