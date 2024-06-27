// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::process;
use dataset_tools::{ walk_rust_files, read_lines };
use anyhow::{ Result, Context };
use log::{ info, warn };
use env_logger;

fn main() -> Result<()> {
    env_logger::init();

    let dir = r"C:\Users\kade\code";
    let mut files_without_warning = Vec::new();

    walk_rust_files(dir, |path, _, _| {
        let lines = read_lines(path).map_err(|e|
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        )?;
        if lines.len() >= 2 {
            let warning_line = "#![warn(clippy::all, clippy::pedantic)]";
            if !lines[0].contains(warning_line) && !lines[1].contains(warning_line) {
                files_without_warning.push(path.to_owned());
            }
        }
        Ok(())
    }).context("Failed to walk through Rust files")?;

    if !files_without_warning.is_empty() {
        warn!("The following files are missing the required warning:");
        for file in &files_without_warning {
            info!("{}", file.display());
        }
        process::exit(1);
    }

    Ok(())
}
