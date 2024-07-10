// This program compresses all executable files in a specified directory (and its subdirectories) using UPX.
//
// The program takes one optional command-line argument: the path to the directory containing the executables.
// If no argument is provided, it defaults to ".\\target\\x86_64-pc-windows-msvc\\release\\".
//
// It recursively iterates over each entry in the specified directory and its subdirectories.
// If an entry is a file with a ".exe" extension, it attempts to compress it using the "upx" command
// with the "--best" option for maximum compression.
//
// If the compression fails for any executable, it prints an error message to the standard error.
//
// This program returns an `std::io::Result<()>`. If an I/O error occurs at any point (such as if the directory
// does not exist or is not readable), the program will return an `Err` variant containing the error.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::path::{ Path, PathBuf };
use tokio::process::Command as AsyncCommand;
use dataset_tools::walk_directory;
use anyhow::{ Context, Result };

async fn compress_exe(path: PathBuf) -> Result<()> {
    println!("Compressing: {}", path.display());
    let status = AsyncCommand::new("upx")
        .arg("--best")
        .arg(&path)
        .status().await
        .context("Failed to run UPX command")?;
    if !status.success() {
        eprintln!("Failed to compress {}", path.display());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_dir = args
        .get(1)
        .map_or("..\\target\\x86_64-pc-windows-msvc\\release\\", String::as_str);

    let target_path = Path::new(target_dir);

    if target_path.is_file() {
        if target_path.extension().map_or(false, |ext| ext == "exe") {
            compress_exe(target_path.to_path_buf()).await?;
        } else {
            println!("The specified file is not an .exe file.");
        }
    } else if target_path.is_dir() {
        walk_directory(target_path, "exe", compress_exe).await?;
    } else {
        println!("The specified path does not exist or is not accessible.");
    }

    Ok(())
}
