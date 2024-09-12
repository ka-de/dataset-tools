// check\src\main.rs

// This program is used to check for different things, it supports looking for rust
// attributes and multiple lines in text files.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use clap::{ Parser, Subcommand };
use dataset_tools::{
    walk_rust_files,
    read_lines,
    walk_directory,
    check_file_for_multiple_lines,
    open_files_in_neovim,
    read_file_content,
    process_rust_file,
    check_pedantic,
    check_optimizations,
    check_attributes,
    RUST_ATTRIBUTES,
};
use regex::Regex;
use crossterm::{ style::{ Color, SetForegroundColor, ResetColor, Stylize }, ExecutableCommand };
use std::{ io, io::stdout, path::{ PathBuf, Path }, sync::Arc, process };
use tokio::sync::Mutex;
use anyhow::{ Result, Context };
use toml::Value;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Attributes {
        #[arg(default_value = ".")]
        directory: String,
    },
    Multiline {
        #[arg(default_value = ".")]
        directory: String,
    },
    Optimizations {
        #[arg(default_value = ".")]
        directory: String,
    },
    Pedantic {
        #[arg(default_value = ".")]
        directory: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Attributes { directory } => check_attributes_command(directory).await?,
        Commands::Multiline { directory } => check_multiline(directory).await?,
        Commands::Optimizations { directory } => check_optimizations_command(directory).await?,
        Commands::Pedantic { directory } => check_pedantic_command(directory).await?,
    }

    Ok(())
}

async fn check_pedantic_command(directory: &str) -> Result<()> {
    let files_without_warning = check_pedantic(directory).await?;

    if !files_without_warning.is_empty() {
        println!("The following files are missing the required warning:");
        for file in files_without_warning.iter() {
            println!("{}", file.display());
        }
        std::process::exit(1);
    } else {
        println!("All Rust files contain the required warning.");
    }

    Ok(())
}

async fn check_optimizations_command(target: &str) -> Result<()> {
    let missing_configs = check_optimizations(target).await?;

    if missing_configs.is_empty() {
        println!("All Cargo.toml files contain the required configurations.");
    } else {
        println!("The following Cargo.toml files are missing the required configurations:");
        for file in missing_configs.iter() {
            println!("{}", file.display());
        }
    }

    Ok(())
}

async fn check_attributes_command(directory: &str) -> Result<()> {
    let matches = check_attributes(directory, RUST_ATTRIBUTES).await?;

    for (path, line_number, line) in matches {
        stdout()
            .execute(SetForegroundColor(Color::Magenta))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        println!("{}:{}", path.display(), line_number);
        stdout()
            .execute(ResetColor)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        println!("{}", line.red());
        println!();
    }

    Ok(())
}

async fn check_multiline(directory: &str) -> Result<()> {
    let multi_line_files = Arc::new(Mutex::new(Vec::new()));

    walk_directory(directory, "txt", |path| {
        let multi_line_files = Arc::clone(&multi_line_files);
        async move {
            if !path.to_str().unwrap_or("").ends_with("-sample-prompts.txt") {
                check_file_for_multiple_lines(path, multi_line_files).await
            } else {
                Ok(())
            }
        }
    }).await.context("Failed to walk directory")?;

    let files = multi_line_files.lock().await;
    if !files.is_empty() {
        println!("\nOpening files with multiple lines in Neovim...");
        open_files_in_neovim(&files).await.context("Failed to open files in Neovim")?;
    } else {
        println!("No files with multiple lines found.");
    }

    Ok(())
}
