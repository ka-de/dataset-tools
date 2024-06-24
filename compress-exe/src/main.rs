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
use std::fs;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 {
        &args[1]
    } else {
        ".\\target\\x86_64-pc-windows-msvc\\release\\"
    };

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("exe") {
            let status = Command::new("upx").arg("--best").arg(path.clone()).status()?;
            if !status.success() {
                eprintln!("Failed to compress {}", path.display());
            }
        }
    }

    Ok(())
}
