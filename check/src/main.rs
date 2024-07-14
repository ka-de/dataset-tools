use clap::{ Parser, Subcommand };
use dataset_tools::{
    walk_rust_files,
    read_lines,
    walk_directory,
    check_file_for_multiple_lines,
    open_files_in_neovim,
};
use regex::Regex;
use crossterm::{ style::{ Color, SetForegroundColor, ResetColor, Stylize }, ExecutableCommand };
use std::{ io, io::stdout, path::PathBuf, sync::Arc };
use tokio::sync::Mutex;
use anyhow::{ Result, Context };

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
    }

    Ok(())
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
