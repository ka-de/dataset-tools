// Turn clippy into a real bitch
#![warn(clippy::all, clippy::pedantic)]

/// This program compresses all executable files in a specified directory using UPX.
///
/// The program takes one optional command-line argument: the path to the directory containing the executables.
/// If no argument is provided, it defaults to ".\\target\\x86_64-pc-windows-msvc\\release\\".
///
/// It iterates over each entry in the specified directory. If an entry is a file with a ".exe" extension,
/// it attempts to compress it using the "upx" command with the "--best" option for maximum compression.
///
/// If the compression fails for any executable, it prints an error message to the standard error.
///
/// This program returns an `std::io::Result<()>`. If an I/O error occurs at any point (such as if the directory
/// does not exist or is not readable), the program will return an `Err` variant containing the error.

use std::env;
use std::process::Command;
use dataset_tools_rs::walk_directory;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_dir = args
        .get(1)
        .map_or(".\\target\\x86_64-pc-windows-msvc\\release\\", String::as_str);

    walk_directory(target_dir.as_ref(), "exe", |path| {
        let status = Command::new("upx").arg("--best").arg(path).status()?;
        if !status.success() {
            eprintln!("Failed to compress {}", path.display());
        }
        Ok(())
    })
}
