// convert-e621-json-to-caption\src\main.rs

//! # e621.net JSON to Caption File Converter
//!
//! This script processes JSON files from e621.net, extracting "post" data to create caption files.
//! It navigates through a directory and its subdirectories, reads each JSON file, and generates
//! a caption file containing the post's rating and tags.
//!
//! Tags that match patterns in `IGNORED_TAGS` are ignored.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ process_json_file, walk_directory, write_to_file };
use regex::Regex;
use serde_json::Value;
use std::{ path::{ Path, PathBuf }, sync::Arc };
use tokio::io;

/// Patterns of tags to be ignored during processing.
const IGNORED_TAGS: [&str; 3] = [
    r"\bconditional_dnp\b",
    r"^\d{4}$", // Years
    r"^\d+:\d+$", // Aspect ratio
];

/// Checks if a tag should be ignored based on predefined patterns.
///
/// # Arguments
///
/// * `tag` - A string slice representing the tag to be checked.
///
/// # Returns
///
/// * `bool` - `true` if the tag matches any pattern in `IGNORED_TAGS`, otherwise `false`.
fn should_ignore_tag(tag: &str) -> bool {
    IGNORED_TAGS.iter().any(|&ignored_tag_pattern| {
        let pattern = Regex::new(ignored_tag_pattern).unwrap();
        pattern.is_match(tag)
    })
}

/// Processes and formats tags from the JSON data.
///
/// # Arguments
///
/// * `tags_dict` - A reference to a JSON Value containing the tags.
///
/// # Returns
///
/// * `Vec<String>` - A vector of strings containing processed and formatted tags.
fn process_tags(tags_dict: &Value) -> Vec<String> {
    let mut processed_tags = Vec::new();
    if let Value::Object(tags) = tags_dict {
        for (category, tags_list) in tags {
            if let Value::Array(tags_array) = tags_list {
                let category_tags: Vec<String> = if category == "artist" {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str())
                        .filter(|&tag| !should_ignore_tag(tag))
                        .map(|tag| format!("by {}", tag.replace('_', " ").replace(" (artist)", "")))
                        .collect()
                } else {
                    tags_array
                        .iter()
                        .filter_map(|tag| tag.as_str())
                        .filter(|&tag| tag.to_lowercase() != "artist" && !should_ignore_tag(tag))
                        .map(|tag| {
                            let tag = tag.replace('_', " ");
                            tag.replace('(', r"\(").replace(')', r"\)")
                        })
                        .collect()
                };
                processed_tags.extend(category_tags);
            }
        }
    }
    processed_tags
}

/// Processes the JSON data and creates a caption file.
///
/// # Arguments
///
/// * `data` - A reference to a JSON Value containing the post data.
/// * `file_path` - A reference to an `Arc<PathBuf>` representing the file path.
///
/// # Returns
///
/// * `io::Result<()>` - The result of the file writing operation.
async fn process_json_data(data: &Value, file_path: &Arc<PathBuf>) -> anyhow::Result<()> {
    if let Some(post) = data.get("post") {
        if let Some(file_data) = post.get("file") {
            if let Some(url) = file_data.get("url").and_then(|u| u.as_str()) {
                let filename = Path::new(url).file_stem().unwrap().to_str().unwrap();
                let caption_path = file_path.with_file_name(format!("{filename}.txt"));

                let rating = post
                    .get("rating")
                    .and_then(|r| r.as_str())
                    .unwrap_or("q");
                let rating_str = match rating {
                    "s" => "safe, ",
                    "e" => "nsfw, ",
                    _ => "questionable, ",
                };

                let mut caption_content = String::from(rating_str);

                if let Some(tags_data) = post.get("tags") {
                    let processed_tags = process_tags(tags_data);
                    if !processed_tags.is_empty() {
                        caption_content.push_str(&processed_tags.join(", "));

                        println!("{}", "-".repeat(50));
                        println!("Caption file: {caption_path:?}");
                        println!("Tags: {caption_content}");
                        println!("{}", "-".repeat(50));
                    }
                }

                write_to_file(&caption_path, &caption_content).await?;
            }
        }
    }
    Ok(())
}

/// Processes a single file.
///
/// # Arguments
///
/// * `file_path` - A `PathBuf` representing the file path.
///
/// # Returns
///
/// * `io::Result<()>` - The result of the file processing operation.
async fn process_file(file_path: PathBuf) -> anyhow::Result<()> {
    println!("Processing file: {file_path:?}");
    let file_path = Arc::new(file_path);

    process_json_file(&file_path, |data| {
        let file_path = Arc::clone(&file_path);
        let data_owned = data.clone();
        async move {
            process_json_data(&data_owned, &file_path).await.map_err(|e|
                io::Error::new(io::ErrorKind::Other, e)
            )
        }
    }).await.map_err(anyhow::Error::from)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root_directory = PathBuf::from(r"E:\training_dir_staging");
    walk_directory(&root_directory, "json", process_file).await?;
    Ok(())
}
