// Turn clippy into a real bitch
#![warn(clippy::all, clippy::pedantic)]

/// This script is used for processing JSON files in a directory and its subdirectories.
/// It reads each JSON file, extracts the "post" data, and creates a caption file with the post's rating and tags.
///
///  The main functions are:
///  - `is_hidden`: Checks if a directory entry is hidden.
///  - `should_ignore_tag`: Checks if a tag should be ignored based on the `IGNORED_TAGS` constant.
///  - `process_tags`: Processes the tags from a JSON object and returns a vector of processed tags.
///  - `process_file`: Processes a single JSON file. It opens the file, reads the JSON data, and creates a caption file with the post's rating and tags.
///  - `recursive_process`: Recursively processes all JSON files in a directory and its subdirectories.
///  - `main`: The entry point of the script. It sets the root directory and calls `recursive_process` to start the processing.
///
///  NOTE: This script uses the `serde_json` library for parsing JSON data,
///        the `regex` library for regular expressions, and the `walkdir`
///        library for walking through directories.

use dataset_tools_rs::{ process_json_file, write_to_file };
use regex::Regex;
use serde_json::Value;
use std::path::{ Path, PathBuf };

const IGNORED_TAGS: [&str; 1] = [r"\bconditional_dnp\b"];

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

fn process_file(file_path: &Path) -> std::io::Result<()> {
    println!("Processing file: {file_path:?}");
    process_json_file(file_path, |data| {
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
                        "s" => "rating_safe, ",
                        "e" => "rating_explicit, ",
                        _ => "rating_questionable, ",
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

                    write_to_file(&caption_path, &caption_content)?;
                }
            }
        }
        Ok(())
    })
}

fn main() -> std::io::Result<()> {
    let root_directory = PathBuf::from(r"E:\training_dir_staging");
    dataset_tools_rs::walk_directory(&root_directory, "json", process_file)
}
