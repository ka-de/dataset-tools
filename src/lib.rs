// src/lib.rs

// Dataset Tools
//
// `dataset_tools` is a collection of building blocks for useful tools for working with various types of data,
// such as datasets, code, images, and other file formats. It provides a set of functions and utilities
// to help with common data processing tasks, including:
//
// - Checking files for multiple lines and opening them in Neovim
// - Formatting text content by replacing multiple spaces with a single space
// - Detecting and skipping hidden directories, Git directories, and build output directories during directory walks
// - Processing Rust files and checking for required compiler warnings
// - Reading and writing JSON files, including formatting and extracting metadata from SafeTensors files
// - Determining if a file is an image and checking if a caption file exists and is not empty
// - Renaming image files to remove the extension
// - Converting JSON files to caption files
// - Deleting files with a specific extension in a directory and its subdirectories
// - Removing letterboxing from image files
//
// This library is designed to be a useful set of tools for working with a variety of data types and formats,
// simplifying common data processing tasks and helping to maintain code quality and consistency.
//
// # Example Usage
//
// ```rust
// use dataset_tools::{
//     walk_rust_files, process_rust_file, format_json_file, process_safetensors_file,
// };
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     // Process Rust files in a directory, checking for the required warning
//     walk_rust_files("src", process_rust_file).await?;
//
//     // Format a JSON file
//     format_json_file("path/to/file.json").await?;
//
//     // Process a SafeTensors file and extract its JSON metadata
//     process_safetensors_file("path/to/file.safetensors").await?;
//
//     Ok(())
// }
// ```

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::{ sync::Arc, path::{ Path, PathBuf } };
use log::{ info, warn };
use walkdir::{ DirEntry, WalkDir };
use serde_json::{ Value, Map };
use anyhow::{ Context, Result };
use memmap2::Mmap;
use safetensors::tensor::SafeTensors;
use image::{ GenericImageView, ImageFormat };
use tokio::{
    sync::Mutex,
    task,
    fs::{ self, File, write },
    io::{ self, AsyncBufReadExt, AsyncWriteExt, BufReader },
    process::Command,
};
use regex::Regex;
use regex::Error as RegexError;

/// Processes a file and adds it to a list if it contains multiple lines.
///
/// # Arguments
///
/// * `path` - A `PathBuf` that holds the path to the file.
/// * `multi_line_files` - An `Arc<Mutex<Vec<PathBuf>>>` that holds the list of files with multiple lines.
///
/// # Returns
///
/// Returns a `Result<()>` indicating the success or failure of the operation.
///
/// # Errors
///
/// This function will return an error if:
/// * The path is invalid.
/// * The file cannot be read.
pub async fn check_file_for_multiple_lines(
    path: PathBuf,
    multi_line_files: Arc<Mutex<Vec<PathBuf>>>
) -> Result<()> {
    let content = read_file_content(path.to_str().context("Invalid path")?).await?;
    let line_count = content.lines().count();

    if line_count > 1 {
        println!("File with multiple lines found: {}", path.display());
        multi_line_files.lock().await.push(path);
    }

    Ok(())
}

/// Opens a list of files in Neovim.
///
/// # Arguments
///
/// * `files` - A slice of `PathBuf` that holds the paths to the files.
///
/// # Returns
///
/// Returns a `Result<()>` indicating the success or failure of the operation.
///
/// # Errors
///
/// This function will return an error if:
/// * Neovim cannot be spawned.
/// * The process cannot wait for Neovim.
pub async fn open_files_in_neovim(files: &[PathBuf]) -> Result<()> {
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

/// Formats the content of a text file by replacing multiple spaces with a single space.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content of the text file.
///
/// # Returns
///
/// Returns a `Result<String, regex::Error>` with the formatted content or an error if the regex could not be compiled.
///
/// # Errors
///
/// This function will return an error if:
/// * The regex cannot be compiled.
#[must_use = "Result must be used to format the content of a text file"]
pub fn format_text_content(content: &str) -> Result<String, RegexError> {
    let space_regex = Regex::new(r"\s+")?;
    Ok(space_regex.replace_all(content, " ").into_owned())
}

/// Checks if a directory entry is the target directory.
///
/// # Returns
///
/// `true` if the entry is a directory and ends with "target", otherwise `false`.
#[must_use = "Determines if the directory entry is a build output directory"]
pub fn is_target_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().ends_with("target")
}

/// Checks if a directory entry is hidden.
///
/// # Returns
///
/// `true` if the entry's file name starts with a dot, indicating it is hidden.
/// Exception: Returns `false` for "." and ".." entries.
#[must_use = "Determines if the directory entry is hidden and should be skipped in directory listings"]
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s != "." && s != ".." && s.starts_with('.'))
}

/// Checks if a directory entry is a git directory.
///
/// # Returns
///
/// `true` if the entry's file name is ".git".
#[must_use = "Determines if the directory entry is a git repository directory"]
pub fn is_git_dir(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy() == ".git"
}

/// Processes a single Rust file and checks for the required warning.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be read.
pub async fn process_rust_file(
    path: &Path,
    files_without_warning: &mut Vec<PathBuf>
) -> io::Result<()> {
    info!("Processing file: {}", path.display());

    let lines = read_lines(path).await?;
    let warning_line = "#![warn(clippy::all, clippy::pedantic)]";

    // Check the first 20 lines for the warning
    let has_warning = lines
        .iter()
        .take(20)
        .any(|line| line.contains(warning_line));

    if !has_warning {
        files_without_warning.push(path.to_owned());
    }
    Ok(())
}

/// Walks through Rust files in a directory and applies a callback function to each file.
/// Skips hidden folders (except "." and ".."), .git folders, and target folders.
///
/// # Errors
///
/// Returns an `io::Error` if a file cannot be opened or read.
pub async fn walk_rust_files<F, Fut>(dir: impl AsRef<Path>, callback: F) -> io::Result<()>
    where F: Fn(PathBuf) -> Fut, Fut: std::future::Future<Output = io::Result<()>>
{
    let walker = WalkDir::new(dir).follow_links(true);

    for entry in walker
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && !is_git_dir(e) && !is_target_dir(e))
        .filter_map(Result::ok) {
        let path = entry.path().to_owned();
        if entry.file_type().is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            callback(path).await?;
        }
    }

    Ok(())
}

/// Reads all lines from a file at the given path.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be opened or read.
#[must_use = "Reads all lines from a file and returns them, requiring handling of the result"]
pub async fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut lines = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        lines.push(line.trim().to_string());
        line.clear();
    }
    Ok(lines)
}

/// Processes a JSON file with a given processor function.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be opened, read, or if the JSON cannot be parsed.
#[must_use = "Processes a JSON file and requires handling of the result to ensure proper file processing"]
pub async fn process_json_file<F, Fut>(file_path: &Path, processor: F) -> io::Result<()>
    where F: FnOnce(&Value) -> Fut, Fut: std::future::Future<Output = io::Result<()>>
{
    let content = fs::read_to_string(file_path).await?;
    let data: Value = serde_json::from_str(&content)?;
    processor(&data).await
}

/// Writes content to a file at the specified path.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be created or written to.
#[must_use = "Writes content to a file and requires handling of the result to ensure data is saved"]
pub async fn write_to_file(path: &Path, content: &str) -> io::Result<()> {
    let mut file = File::create(path).await?;
    file.write_all(content.as_bytes()).await
}

/// Walks through a directory and applies a callback function to each file with the specified extension.
///
/// # Errors
///
/// Returns an `io::Error` if there's an issue with directory traversal or file operations.
#[must_use = "Walks through a directory and requires handling of the result to ensure proper file processing"]
pub async fn walk_directory<F, Fut>(
    dir: impl AsRef<Path>,
    extension: &str,
    callback: F
)
    -> Result<()>
    where F: Fn(PathBuf) -> Fut, Fut: std::future::Future<Output = Result<()>> + Send + 'static
{
    let dir = dir.as_ref();
    info!("Starting directory walk in: {dir:?}");

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| {
            let is_hidden = !is_hidden(e);
            let is_git_dir = !is_git_dir(e);
            info!("Entry: {e:?}, is_hidden: {is_hidden}, is_git_dir: {is_git_dir}");
            is_hidden && is_git_dir
        })
        .filter_map(Result::ok) {
        let path = entry.path().to_owned();
        info!("Processing path: {path:?}");
        if path.extension().map_or(false, |ext| ext == extension) {
            info!("Path matches extension: {path:?}");
            let path_clone = path.clone();
            if let Err(e) = task::spawn(callback(path)).await? {
                warn!("Error processing file: {path_clone:?}. Error: {e}");
            }
        }
    }

    info!("Finished directory walk in: {dir:?}");
    Ok(())
}

/// Retrieves JSON metadata from a buffer.
///
/// # Errors
///
/// Returns an error if the metadata cannot be read or parsed.
#[must_use = "Retrieves JSON metadata and requires handling of the result to ensure metadata is obtained"]
pub async fn get_json_metadata(path: &Path) -> Result<Value> {
    let file = File::open(path).await?;
    let mmap = unsafe { Mmap::map(&file)? };

    let (_header_size, metadata) = task
        ::spawn_blocking(move || { SafeTensors::read_metadata(&mmap) }).await
        .context("Cannot read metadata")??;

    let metadata = metadata.metadata().as_ref().context("No metadata available")?;

    let mut kv = Map::with_capacity(metadata.len());
    for (key, value) in metadata {
        let json_value = serde_json::from_str(value).unwrap_or_else(|_| {
            match value.as_str() {
                "True" => Value::Bool(true),
                "False" => Value::Bool(false),
                "None" => Value::Null,
                s => Value::String(s.into()),
            }
        });
        kv.insert(key.clone(), json_value);
    }
    Ok(Value::Object(kv))
}

/// Processes a `SafeTensors` file and extracts its JSON metadata.
///
/// # Errors
///
/// Returns an error if the file cannot be opened, read, or processed.
#[must_use = "Processes a SafeTensors file and requires handling of the result to ensure metadata is extracted"]
pub async fn process_safetensors_file(path: &Path) -> Result<()> {
    let json = get_json_metadata(path).await?;
    let pretty_json = serde_json::to_string_pretty(&json)?;
    info!("{pretty_json}");
    write(path.with_extension("json"), pretty_json).await?;
    Ok(())
}

/// Determines if the given path is an image file.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "jxl"),
        None => false,
    }
}

/// Checks if a caption file exists and is not empty.
#[must_use = "Checks if the caption file exists and is not empty and the result should be checked"]
pub async fn caption_file_exists_and_not_empty(path: &Path) -> bool {
    if path.exists() {
        match fs::read_to_string(path).await {
            Ok(content) => !content.trim().is_empty(),
            Err(_) => false,
        }
    } else {
        false
    }
}

/// Formats a JSON file to have pretty-printed JSON.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be read, parsed as JSON, or written back.
#[must_use = "Formats a JSON file and requires handling of the result to ensure the file is properly formatted"]
pub async fn format_json_file(path: PathBuf) -> Result<()> {
    info!("Processing file: {}", path.display());

    let file_content = fs
        ::read_to_string(path.clone()).await
        .context("Failed to read file content")?;
    let json: Value = serde_json::from_str(&file_content).context("Failed to parse JSON")?;
    let pretty_json = serde_json::to_string_pretty(&json).context("Failed to format JSON")?;
    fs::write(path.clone(), pretty_json).await.context("Failed to write formatted JSON")?;

    info!("Formatted {} successfully.", path.display());
    Ok(())
}

/// Reads the content of a file.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be opened or read.
#[must_use = "Reads the content of a file and requires handling of the result to ensure the content is retrieved"]
pub async fn read_file_content(file: &str) -> io::Result<String> {
    fs::read_to_string(file).await
}

/// Splits content into tags and sentences.
#[must_use = "Splits content into tags and sentences and the result should be checked"]
pub fn split_content(content: &str) -> (Vec<&str>, &str) {
    let split: Vec<_> = content.split("., ").collect();
    let tags: Vec<_> = split[0].split(',').collect();
    let sentences = split.get(1).unwrap_or(&"");
    (tags, sentences.trim())
}

/// Renames a file to remove the image extension.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be renamed.
#[must_use = "Renames a file and requires handling of the result to ensure the file is properly renamed"]
pub async fn rename_file_without_image_extension(path: &Path) -> io::Result<()> {
    if let Some(old_name) = path.to_str() {
        if old_name.contains(".jpeg") || old_name.contains(".png") || old_name.contains(".jpg") {
            let new_name = old_name.replace(".jpeg", "").replace(".png", "").replace(".jpg", "");
            fs::rename(old_name, &new_name).await?;
            info!("Renamed {old_name} to {new_name}");
        }
    }
    Ok(())
}

/// Processes a JSON file and converts it to a caption file.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be read, parsed, or written.
#[must_use = "Processes a JSON file to create a caption file and requires handling of the result to ensure proper conversion"]
pub async fn process_json_to_caption(input_path: &Path) -> io::Result<()> {
    if input_path.extension().and_then(|s| s.to_str()) == Some("json") {
        let content = fs::read_to_string(input_path).await?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json {
            let mut tags: Vec<(String, f64)> = map
                .iter()
                .filter_map(|(key, value)| {
                    if let Value::Number(num) = value {
                        let probability = num.as_f64().unwrap_or(0.0);
                        if probability > 0.2 {
                            Some((key.replace('(', "\\(").replace(')', "\\)"), probability))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            tags.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let output_path = input_path.with_extension("txt");
            let mut output_file = File::create(output_path).await?;
            output_file.write_all(
                tags
                    .iter()
                    .map(|(tag, _)| tag.clone())
                    .collect::<Vec<String>>()
                    .join(", ")
                    .as_bytes()
            ).await?;
        }
    }
    Ok(())
}

/// Deletes files with a specific extension in a directory and its subdirectories.
///
/// # Errors
///
/// Returns an `io::Error` if there's an issue with file operations.
#[must_use = "Deletes files with a specific extension and requires handling of the result to ensure proper file deletion"]
pub async fn delete_files_with_extension(target_dir: &Path, extension: &str) -> io::Result<()> {
    let mut tasks = Vec::new();

    for entry in WalkDir::new(target_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path().to_owned();
        if path.is_file() {
            if let Some(file_extension) = path.extension() {
                if file_extension.eq_ignore_ascii_case(extension) {
                    tasks.push(
                        tokio::spawn(async move {
                            if let Err(e) = fs::remove_file(&path).await {
                                eprintln!("Failed to remove {}: {e}", path.display());
                            } else {
                                println!("Removed: {}", path.display());
                            }
                        })
                    );
                }
            }
        }
    }

    for task in tasks {
        task.await?;
    }

    Ok(())
}

/// Processes an image file to remove letterboxing.
///
/// # Errors
///
/// Returns an `io::Error` if there's an issue with image processing or file operations.
#[must_use = "Processes an image to remove letterboxing and requires handling of the result to ensure proper image modification"]
pub async fn remove_letterbox(input_path: &Path) -> io::Result<()> {
    // Handle all image formats through image crate
    let img_bytes = fs::read(input_path).await?;
    let img = image::load_from_memory(&img_bytes)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let (width, height) = img.dimensions();

    let mut top = 0;
    let mut bottom = height - 1;
    let mut left = 0;
    let mut right = width - 1;

    // Find top
    'outer: for y in 0..height {
        for x in 0..width {
            if img.get_pixel(x, y)[0] != 0 {
                top = y;
                break 'outer;
            }
        }
    }

    // Find bottom
    'outer: for y in (0..height).rev() {
        for x in 0..width {
            if img.get_pixel(x, y)[0] != 0 {
                bottom = y;
                break 'outer;
            }
        }
    }

    // Find left
    'outer: for x in 0..width {
        for y in 0..height {
            if img.get_pixel(x, y)[0] != 0 {
                left = x;
                break 'outer;
            }
        }
    }

    // Find right
    'outer: for x in (0..width).rev() {
        for y in 0..height {
            if img.get_pixel(x, y)[0] != 0 {
                right = x;
                break 'outer;
            }
        }
    }

    let cropped = img.crop_imm(left, top, right - left + 1, bottom - top + 1);
    let mut buf = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(input_path, buf).await?;

    Ok(())
}

pub const RUST_ATTRIBUTES: &[&str] = &[
    "cfg",
    "cfg_attr",
    "test",
    "ignore",
    "should_panic",
    "automatically_derived",
    "macro_export",
    "macro_use",
    "proc_macro",
    "proc_macro_derive",
    "proc_macro_attribute",
    "allow",
    "warn",
    "deny",
    "forbid",
    "deprecated",
    "diagnostic::on_unimplemented",
    "link",
    "link_name",
    "link_ordinal",
    "no_link",
    "repr",
    "crate_type",
    "no_main",
    "export_name",
    "link_section",
    "no_mangle",
    "used",
    "crate_name",
    "inline",
    "cold",
    "no_builtins",
    "target_feature",
    "track_caller",
    "instruction_set",
    "doc",
    "no_std",
    "no_implicit_prelude",
    "path",
    "recursion_limit",
    "type_length_limit",
    "panic_handler",
    "global_allocator",
    "windows_subsystem",
    "feature",
    "non_exhaustive",
    "debugger_visualizer",
];

pub async fn check_pedantic(directory: &str) -> Result<Vec<PathBuf>> {
    let files_without_warning = Arc::new(Mutex::new(Vec::new()));

    let target = PathBuf::from(directory);
    let canonical_target = target.canonicalize().context("Failed to canonicalize path")?;

    if canonical_target.is_file() && canonical_target.extension().map_or(false, |ext| ext == "rs") {
        let mut guard = files_without_warning.lock().await;
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
        return Err(anyhow::anyhow!("Invalid target. Please provide a .rs file or a directory."));
    }

    Ok(Arc::try_unwrap(files_without_warning).unwrap().into_inner())
}

pub async fn check_optimizations(target: &str) -> Result<Vec<PathBuf>> {
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
        return Err(anyhow::anyhow!("Invalid path: {}", target_path.display()));
    }

    Ok(Arc::try_unwrap(missing_configs).unwrap().into_inner())
}

pub async fn check_cargo_toml(path: &Path) -> Result<bool> {
    let content = read_file_content(path.to_str().unwrap()).await.context("Failed to read file")?;
    let toml_value: Value = content.parse().context("Failed to parse TOML")?;

    let Some(profile) = toml_value.get("profile") else {
        return Ok(false);
    };

    // Check [profile.dev]
    let Some(dev) = profile.get("dev") else {
        return Ok(false);
    };
    if dev.get("opt-level").and_then(|v| v.as_i64()) != Some(3) {
        return Ok(false);
    }

    // Check [profile.dev.package."*"]
    let Some(dev_package) = dev.get("package").and_then(|p| p.get("*")) else {
        return Ok(false);
    };
    if
        dev_package.get("opt-level").and_then(|v| v.as_i64()) != Some(3) ||
        dev_package.get("codegen-units").and_then(|v| v.as_i64()) != Some(1)
    {
        return Ok(false);
    }

    // Check [profile.release]
    let Some(release) = profile.get("release") else {
        return Ok(false);
    };
    if
        release.get("opt-level").and_then(|v| v.as_i64()) != Some(3) ||
        release.get("lto").and_then(|v| v.as_bool()) != Some(true) ||
        release.get("codegen-units").and_then(|v| v.as_i64()) != Some(1) ||
        release.get("strip").and_then(|v| v.as_bool()) != Some(true)
    {
        return Ok(false);
    }

    Ok(true)
}

pub async fn check_attributes(
    directory: &str,
    attributes: &[&str]
) -> Result<Vec<(PathBuf, usize, String)>> {
    let re = Arc::new(
        Regex::new(
            &format!(r"#\[\s*({})|#!\[\s*({})\]", attributes.join("|"), attributes.join("|"))
        ).context("Failed to create regex")?
    );
    let matches = Arc::new(Mutex::new(Vec::new()));

    walk_rust_files(directory, {
        let re = Arc::clone(&re);
        let matches = Arc::clone(&matches);
        move |path: PathBuf| {
            let re = Arc::clone(&re);
            let matches = Arc::clone(&matches);
            async move {
                let lines = read_lines(&path).await?;
                for (line_number, line) in lines.iter().enumerate() {
                    if re.is_match(line) {
                        matches
                            .lock().await
                            .push((path.clone(), line_number + 1, line.to_string()));
                    }
                }
                Ok(())
            }
        }
    }).await.context("Failed to walk rust files")?;

    Ok(Arc::try_unwrap(matches).unwrap().into_inner())
}
