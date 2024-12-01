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
    is_image_file,
    caption_file_exists_and_not_empty,
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
    Pedantic {
        #[arg(default_value = ".")]
        directory: String,
    },
    EmptyCaptions {
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
        Commands::Pedantic { directory } => check_pedantic(directory).await?,
        Commands::EmptyCaptions { directory } => check_empty_captions(directory).await?,
    }

    Ok(())
}

async fn check_pedantic(directory: &str) -> Result<Vec<PathBuf>> {
    let files_without_warning = Arc::new(Mutex::new(Vec::new()));

    let target = PathBuf::from(directory);
    let canonical_target = target.canonicalize().context("Failed to canonicalize path")?;

    if canonical_target.is_file() && canonical_target.extension().map_or(false, |ext| ext == "rs") {
        let files_without_warning_clone = Arc::clone(&files_without_warning);
        let mut guard = files_without_warning_clone.lock().await;
        process_rust_file(&canonical_target, &mut *guard).await?;
    } else if canonical_target.is_dir() {
        walk_rust_files(&canonical_target, |path| {
            let files_without_warning_clone = Arc::clone(&files_without_warning);
            let path_buf = path.to_path_buf();
            async move {
                let mut guard = files_without_warning_clone.lock().await;
                process_rust_file(&path_buf, &mut *guard).await
            }
        }).await.context("Failed to walk through Rust files")?;
    } else {
        println!("Invalid target. Please provide a .rs file or a directory.");
        std::process::exit(1);
    }

    let files_without_warning = files_without_warning.lock().await;
    if !files_without_warning.is_empty() {
        println!("The following files are missing the required warning:");
        for file in files_without_warning.iter() {
            println!("{}", file.display());
        }
        std::process::exit(1);
    } else {
        println!("All Rust files contain the required warning.");
    }

    Ok(files_without_warning.clone())
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

    let found_attributes = Arc::new(Mutex::new(Vec::new()));
    let found_attributes_clone = Arc::clone(&found_attributes);

    walk_rust_files(directory, move |path: PathBuf| {
        let re = Arc::clone(&re);
        let found_attributes = Arc::clone(&found_attributes_clone);
        async move {
            let lines = read_lines(&path).await?;
            for (line_number, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    found_attributes.lock().await.push((
                        path.clone(),
                        line_number + 1,
                        line.to_string(),
                    ));
                    
                    // Still print for CLI usage
                    stdout()
                        .execute(SetForegroundColor(Color::Magenta))
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                    println!("{}:{}", path.display(), line_number + 1);
                    stdout()
                        .execute(ResetColor)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

                    let start = line_number.saturating_sub(3);
                    let end = (line_number + 2).min(lines.len());
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

    let attributes = found_attributes.lock().await;
    // Print or process attributes as needed
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

async fn check_empty_captions(directory: &str) -> Result<()> {
    let empty_captions = Arc::new(Mutex::new(Vec::new()));

    walk_directory(directory, "jpg", |path| {
        let empty_captions = Arc::clone(&empty_captions);
        async move {
            if is_image_file(&path) {
                let caption_path = path.with_extension("txt");
                if !caption_file_exists_and_not_empty(&caption_path).await {
                    empty_captions.lock().await.push(path);
                }
            }
            Ok(())
        }
    }).await.context("Failed to walk directory")?;

    let files = empty_captions.lock().await;
    if !files.is_empty() {
        println!("The following image files have empty or missing captions:");
        for file in files.iter() {
            println!("{}", file.display());
        }
    } else {
        println!("No image files with empty or missing captions found.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<PathBuf> {
        let path = dir.join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    #[tokio::test]
    async fn test_check_pedantic() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test files
        let file_with_warning = create_test_file(
            temp_dir.path(),
            "with_warning.rs",
            "#![warn(clippy::all, clippy::pedantic)]\nfn main() {}"
        ).await.unwrap();

        let file_without_warning = create_test_file(
            temp_dir.path(),
            "without_warning.rs",
            "fn main() {}"
        ).await.unwrap();

        // Test directory with mixed files
        let result = check_pedantic(temp_dir.path().to_str().unwrap()).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], file_without_warning);

        // Test single file with warning
        let result = check_pedantic(file_with_warning.to_str().unwrap()).await.unwrap();
        assert!(result.is_empty());

        // Test single file without warning
        let result = check_pedantic(file_without_warning.to_str().unwrap()).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_check_optimizations() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create test Cargo.toml files
        let optimized_toml = create_test_file(
            temp_dir.path(),
            "Cargo.toml",
            r#"
[profile.dev]
opt-level = 3

[profile.dev.package."*"]
opt-level = 3
codegen-units = 1

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
            "#
        ).await.unwrap();

        let unoptimized_toml = create_test_file(
            &temp_dir.path().join("subdir"),
            "Cargo.toml",
            "[package]\nname = \"test\"\nversion = \"0.1.0\""
        ).await.unwrap();

        // Test directory with both files
        check_optimizations(temp_dir.path().to_str().unwrap()).await.unwrap();
    }

    #[tokio::test]
    async fn test_check_attributes() {
        let temp_dir = TempDir::new().unwrap();
        
        let file_with_attrs = create_test_file(
            temp_dir.path(),
            "with_attrs.rs",
            r#"
#[derive(Debug)]
#[cfg(test)]
struct Test {}
            "#
        ).await.unwrap();

        check_attributes(temp_dir.path().to_str().unwrap()).await.unwrap();
    }
}
