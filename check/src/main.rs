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
};
use regex::Regex;
use crossterm::{ style::{ Color, SetForegroundColor, ResetColor, Stylize }, ExecutableCommand };
use std::{ io, io::stdout, path::{ PathBuf, Path }, sync::Arc };
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
}

// List of built-in attributes in Rust
#[rustfmt::skip]
const ATTRIBUTES: &[&str] = &[
    "cfg", "cfg_attr", "test", "ignore", "should_panic", //"derive",
    "automatically_derived", "macro_export", "macro_use", "proc_macro",
    "proc_macro_derive", "proc_macro_attribute", "allow", "warn",
    "deny", "forbid", "deprecated", //"must_use",
    "diagnostic::on_unimplemented", "link", "link_name", "link_ordinal",
    "no_link", "repr", "crate_type", "no_main", "export_name", "link_section",
    "no_mangle", "used", "crate_name", "inline", "cold", "no_builtins",
    "target_feature", "track_caller", "instruction_set", "doc", "no_std",
    "no_implicit_prelude", "path", "recursion_limit", "type_length_limit",
    "panic_handler", "global_allocator", "windows_subsystem",
	 "feature", "non_exhaustive", "debugger_visualizer", // "tokio::main",
];

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Attributes { directory } => check_attributes(directory).await?,
        Commands::Multiline { directory } => check_multiline(directory).await?,
        Commands::Optimizations { directory } => check_optimizations(directory).await?,
    }

    Ok(())
}

async fn check_optimizations(target: &str) -> Result<()> {
    let target_path = Path::new(target);
    let missing_configs = Arc::new(Mutex::new(Vec::new()));

    if target_path.is_file() && target_path.file_name().unwrap() == "Cargo.toml" {
        if !check_cargo_toml(target_path).await.unwrap_or(false) {
            missing_configs.lock().await.push(target_path.to_owned());
        }
    } else if target_path.is_dir() {
        walk_directory(target_path, "toml", |path: PathBuf| {
            let missing_configs = Arc::clone(&missing_configs);
            async move {
                if
                    path.file_name().unwrap() == "Cargo.toml" &&
                    !check_cargo_toml(&path).await.unwrap_or(false)
                {
                    missing_configs.lock().await.push(path);
                }
                Ok(())
            }
        }).await?;
    } else {
        println!("Invalid path: {}", target_path.display());
        return Ok(());
    }

    let missing_configs = missing_configs.lock().await;
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

async fn check_cargo_toml(path: &Path) -> Result<bool> {
    let content = read_file_content(path.to_str().unwrap()).await.context("Failed to read file")?;
    let toml_value: Value = content.parse().context("Failed to parse TOML")?;

    let Some(profile) = toml_value.get("profile") else {
        return Ok(false);
    };

    // Check [profile.dev]
    let Some(dev) = profile.get("dev") else {
        return Ok(false);
    };
    if dev.get("opt-level") != Some(&Value::Integer(3)) {
        return Ok(false);
    }

    // Check [profile.dev.package."*"]
    let Some(dev_package) = dev.get("package").and_then(|p| p.get("*")) else {
        return Ok(false);
    };
    if
        dev_package.get("opt-level") != Some(&Value::Integer(3)) ||
        dev_package.get("codegen-units") != Some(&Value::Integer(1))
    {
        return Ok(false);
    }

    // Check [profile.release]
    let Some(release) = profile.get("release") else {
        return Ok(false);
    };
    if
        release.get("opt-level") != Some(&Value::Integer(3)) ||
        release.get("lto") != Some(&Value::Boolean(true)) ||
        release.get("codegen-units") != Some(&Value::Integer(1)) ||
        release.get("strip") != Some(&Value::Boolean(true))
    {
        return Ok(false);
    }

    Ok(true)
}

async fn check_attributes(directory: &str) -> Result<()> {
    let re = Arc::new(
        Regex::new(
            &format!(r"#\[\s*({})|#!\[\s*({})\]", ATTRIBUTES.join("|"), ATTRIBUTES.join("|"))
        ).context("Failed to create regex")?
    );

    walk_rust_files(directory, move |path: PathBuf| {
        let re = Arc::clone(&re);
        async move {
            let lines = read_lines(&path).await?;
            for (line_number, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    let start = line_number.saturating_sub(3);
                    let end = (line_number + 2).min(lines.len());

                    stdout()
                        .execute(SetForegroundColor(Color::Magenta))
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    println!("{}:{}", path.display(), line_number + 1);
                    stdout()
                        .execute(ResetColor)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    for (i, line) in lines[start..end].iter().enumerate() {
                        if i + start == line_number {
                            let highlighted = re.replace_all(line, |caps: &regex::Captures| {
                                format!("{}", caps[0].red())
                            });
                            println!("{highlighted}");
                        } else {
                            println!("{line}");
                        }
                    }
                    println!();
                }
            }
            Ok(())
        }
    }).await.context("Failed to walk rust files")?;

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
