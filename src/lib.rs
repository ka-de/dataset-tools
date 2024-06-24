// lib.rs

use std::fs::{ self, File };
use std::io::{ self, BufRead, Write };
use std::path::Path;
use walkdir::{ DirEntry, WalkDir };
use serde_json::Value;
use anyhow::{ Context, Result };
use memmap2::Mmap;
use safetensors::tensor::SafeTensors;

pub fn is_target_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().ends_with("target")
}

pub fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s.starts_with('.'))
}

pub fn is_not_git(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy() != ".git"
}

pub fn walk_rust_files<F>(dir: &str, mut callback: F) -> io::Result<()>
    where F: FnMut(&Path, usize, &str) -> io::Result<()>
{
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_target_dir(e))
        .filter_map(Result::ok) {
        if let Some(path) = entry.path().to_str() {
            if path.ends_with(".rs") {
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

pub fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect()
}

pub fn process_json_file<F>(file_path: &Path, processor: F) -> io::Result<()>
    where F: Fn(&Value) -> io::Result<()>
{
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;
    processor(&data)
}

pub fn write_to_file(path: &Path, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())
}

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

pub fn process_safetensors_file(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let json = get_json_metadata(&mmap)?;
    let pretty_json = serde_json::to_string_pretty(&json)?;

    println!("{pretty_json}");
    write_to_file(&path.with_extension("json"), &pretty_json)?;
    Ok(())
}

pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png"),
        None => false,
    }
}

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

pub fn read_file_content(file: &str) -> io::Result<String> {
    let file = File::open(file)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect::<Result<String, _>>()
}

pub fn split_content(content: &str) -> (Vec<&str>, &str) {
    let split: Vec<_> = content.split("., ").collect();
    let tags: Vec<_> = split[0].split(',').collect();
    let sentences = split.get(1).unwrap_or(&"");
    (tags, sentences.trim())
}

pub fn rename_file_without_image_extension(path: &Path) -> io::Result<()> {
    if let Some(old_name) = path.to_str() {
        if old_name.contains(".jpeg") || old_name.contains(".png") || old_name.contains(".jpg") {
            let new_name = old_name.replace(".jpeg", "").replace(".png", "").replace(".jpg", "");
            fs::rename(old_name, &new_name)?;
            println!("Renamed {} to {}", old_name, new_name);
        }
    }
    Ok(())
}
