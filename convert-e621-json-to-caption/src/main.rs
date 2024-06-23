/// * This script is used for processing JSON files in a directory and its subdirectories.
///  * It reads each JSON file, extracts the "post" data, and creates a caption file with the post's rating and tags.
///  *
///  * The main functions are:
///  * - `is_hidden`: Checks if a directory entry is hidden.
///  * - `should_ignore_tag`: Checks if a tag should be ignored based on the `IGNORED_TAGS` constant.
///  * - `process_tags`: Processes the tags from a JSON object and returns a vector of processed tags.
///  * - `process_file`: Processes a single JSON file. It opens the file, reads the JSON data, and creates a caption file with the post's rating and tags.
///  * - `recursive_process`: Recursively processes all JSON files in a directory and its subdirectories.
///  * - `main`: The entry point of the script. It sets the root directory and calls `recursive_process` to start the processing.
///  *
///  * Note: This script uses the `serde_json` library for parsing JSON data, the `regex` library for regular expressions, and the `walkdir` library for walking through directories.

use std::fs::File;
use std::io::{ BufReader, Write };
use std::path::{ Path, PathBuf };
use regex::Regex;
use serde_json::Value;
use walkdir::{ WalkDir, DirEntry };

/// Define tags to be ignored using regular expressions for exact matching
const IGNORED_TAGS: [&str; 1] = [r"\bconditional_dnp\b"];

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn should_ignore_tag(tag: &str) -> bool {
    IGNORED_TAGS.iter().any(|&ignored_tag_pattern| {
        let pattern = Regex::new(ignored_tag_pattern).unwrap();
        pattern.is_match(tag)
    })
}

fn process_tags(tags_dict: &Value) -> Vec<String> {
    let mut processed_tags = Vec::new();
    if let Value::Object(tags) = tags_dict {
        for (category, tags_list) in tags {
            if let Value::Array(tags_array) = tags_list {
                let category_tags: Vec<String> = if category == "artist" {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str())
                        .filter(|&tag| !should_ignore_tag(tag)) // Apply filter here
                        .map(|tag| format!("by {}", tag.replace("_", " ").replace(" (artist)", "")))
                        .collect()
                } else {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str())
                        .filter(|&tag| tag.to_lowercase() != "artist" && !should_ignore_tag(tag))
                        .map(|tag| {
                            let tag = tag.replace("_", " ");
                            tag.replace("(", r"\(").replace(")", r"\)")
                        })
                        .collect()
                };
                processed_tags.extend(category_tags);
            }
        }
    }
    processed_tags
}

fn process_file(file_path: &Path) -> std::io::Result<()> {
    println!("Processing file: {:?}", file_path);
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data: Value = serde_json::from_reader(reader)?;

    if let Some(post) = data.get("post") {
        if let Some(file_data) = post.get("file") {
            if let Some(url) = file_data.get("url").and_then(|u| u.as_str()) {
                let filename = Path::new(url).file_stem().unwrap().to_str().unwrap();
                let caption_file = format!("{}.txt", filename);
                let caption_path = file_path.with_file_name(caption_file);

                let mut file = File::create(&caption_path)?;

                let rating = post
                    .get("rating")
                    .and_then(|r| r.as_str())
                    .unwrap_or("q");
                let rating_str = match rating {
                    "s" => "rating_safe, ",
                    "e" => "rating_explicit, ",
                    _ => "rating_questionable, ",
                };
                file.write_all(rating_str.as_bytes())?;

                if let Some(tags_data) = post.get("tags") {
                    let processed_tags = process_tags(tags_data);
                    if !processed_tags.is_empty() {
                        let tags_line = processed_tags.join(", ");
                        file.write_all(tags_line.as_bytes())?;

                        println!("{}", "-".repeat(50));
                        println!("Caption file: {:?}", caption_path);
                        println!("Tags: {}", tags_line);
                        println!("{}", "-".repeat(50));
                    }
                }
            }
        }
    }

    Ok(())
}

fn recursive_process(directory: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_entry(|e| !is_hidden(e) && e.file_name() != ".git")
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            process_file(path)?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let root_directory = PathBuf::from(r"E:\training_dir_staging");
    recursive_process(&root_directory)
}
