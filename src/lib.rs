// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

// lib.rs

use std::fs::{ self, File };
use std::io::{ self, BufRead, Write };
use std::path::Path;
use walkdir::{ DirEntry, WalkDir };
use serde_json::Value;
use anyhow::{ Context, Result };
use memmap2::Mmap;
use safetensors::tensor::SafeTensors;

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
#[must_use = "Determines if the directory entry is hidden and should be skipped in directory listings"]
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s.starts_with('.'))
}

/// Checks if a directory entry is not a git directory.
///
/// # Returns
///
/// `true` if the entry's file name is not ".git".
#[must_use = "Determines if the directory entry is not a git repository directory"]
pub fn is_not_git(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy() != ".git"
}

/// Walks through Rust files in a directory and applies a callback function to each line.
///
/// # Errors
///
/// Returns an `io::Error` if a file cannot be opened or read.
#[must_use = "Executes a callback on each line of Rust files and requires handling of the result"]
pub fn walk_rust_files<F>(dir: &str, mut callback: F) -> io::Result<()>
    where F: FnMut(&Path, usize, &str) -> io::Result<()>
{
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_target_dir(e))
        .filter_map(Result::ok) {
        if let Some(path) = entry.path().to_str() {
            if
                std::path::Path
                    ::new(path)
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("rs"))
            {
                let file = File::open(entry.path())?;
                let reader = io::BufReader::new(file);

                for (i, line) in reader.lines().enumerate() {
                    let line = line?;
                    callback(entry.path(), i + 1, &line)?;
                }
            }
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
pub fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect()
}

/// Processes a JSON file with a given processor function.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be opened, read, or if the JSON cannot be parsed.
#[must_use = "Processes a JSON file and requires handling of the result to ensure proper file processing"]
pub fn process_json_file<F>(file_path: &Path, processor: F) -> io::Result<()>
    where F: Fn(&Value) -> io::Result<()>
{
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;
    processor(&data)
}

/// Writes content to a file at the specified path.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be created or written to.
#[must_use = "Writes content to a file and requires handling of the result to ensure data is saved"]
pub fn write_to_file(path: &Path, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())
}

/// Walks through a directory and applies a callback function to each file with the specified extension.
///
/// # Errors
///
/// Returns an `io::Error` if there's an issue with directory traversal or file operations.
#[must_use = "Walks through a directory and requires handling of the result to ensure proper file processing"]
pub fn walk_directory<F>(directory: &Path, file_extension: &str, mut callback: F) -> io::Result<()>
    where F: FnMut(&Path) -> io::Result<()>
{
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && is_not_git(e))
        .filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == file_extension) {
            callback(path)?;
        }
    }
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
pub fn process_safetensors_file(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let json = get_json_metadata(&mmap)?;
    let pretty_json = serde_json::to_string_pretty(&json)?;

    println!("{pretty_json}");
    write_to_file(&path.with_extension("json"), &pretty_json)?;
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
pub fn caption_file_exists_and_not_empty(path: &Path) -> bool {
    if path.exists() {
        match fs::read_to_string(path) {
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
pub fn format_json_file(path: &Path) -> io::Result<()> {
    println!("Processing file: {}", path.display());

    let file_content = fs::read_to_string(path)?;
    let json: Value = serde_json
        ::from_str(&file_content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let pretty_json = serde_json
        ::to_string_pretty(&json)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, pretty_json)?;

    println!("Formatted {} successfully.", path.display());
    Ok(())
}

/// Reads the content of a file.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be opened or read.
#[must_use = "Reads the content of a file and requires handling of the result to ensure the content is retrieved"]
pub fn read_file_content(file: &str) -> io::Result<String> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect::<Result<String, _>>()
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
pub fn rename_file_without_image_extension(path: &Path) -> io::Result<()> {
    if let Some(old_name) = path.to_str() {
        if old_name.contains(".jpeg") || old_name.contains(".png") || old_name.contains(".jpg") {
            let new_name = old_name.replace(".jpeg", "").replace(".png", "").replace(".jpg", "");
            fs::rename(old_name, &new_name)?;
            println!("Renamed {old_name} to {new_name}");
        }
    }
    Ok(())
}
