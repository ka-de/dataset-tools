// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

// src\lib.rs
extern crate image;

use std::path::{ Path, PathBuf };
use log::{ info, warn };
use tokio::task;
use walkdir::{ DirEntry, WalkDir };
use serde_json::Value;
use anyhow::{ Context, Result };
use memmap2::Mmap;
use safetensors::tensor::SafeTensors;
use image::GenericImageView;
use tokio::fs::{ self, File };
use tokio::io::{ self, AsyncBufReadExt, AsyncWriteExt, BufReader };

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
fn is_git_dir(entry: &DirEntry) -> bool {
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
pub fn get_json_metadata(buffer: &[u8]) -> Result<Value> {
    let (_header_size, metadata) =
        SafeTensors::read_metadata(buffer).context("Cannot read metadata")?;
    let metadata = metadata.metadata().as_ref().context("No metadata available")?;

    let mut kv = serde_json::Map::with_capacity(metadata.len());
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
    let file = File::open(path).await?;
    let mmap = unsafe { Mmap::map(&file)? };

    match get_json_metadata(&mmap) {
        Ok(json) => {
            let pretty_json = serde_json::to_string_pretty(&json)?;
            info!("{pretty_json}");
            write_to_file(&path.with_extension("json"), &pretty_json).await?;
        }
        Err(e) => {
            warn!("No metadata found for file: {:?}. Error: {}", path, e);
        }
    }
    Ok(())
}

/// Determines if the given path is an image file.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png"),
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
                                eprintln!("Failed to remove {}: {}", path.display(), e);
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
pub fn remove_letterbox(input_path: &Path) -> io::Result<()> {
    let mut img = image::open(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
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

    let cropped = img.crop(left, top, right - left + 1, bottom - top + 1);
    cropped.save(input_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}
