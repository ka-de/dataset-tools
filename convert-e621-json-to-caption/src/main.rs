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
use std::env;

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
    println!("Starting process_tags");
    let mut processed_tags = Vec::new();
    
    if let Value::Object(tags) = tags_dict {
        println!("Found tags object with {} categories", tags.len());
        
        for (category, tags_list) in tags {
            println!("Processing category: {}", category);
            
            if let Value::Array(tags_array) = tags_list {
                println!("Found {} tags in category", tags_array.len());
                
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
                            tag.replace('_', " ")
                            // tag.replace('(', r"\(").replace(')', r"\)")
                        })
                        .collect()
                };
                
                println!("Processed {} tags in category", category_tags.len());
                processed_tags.extend(category_tags);
            } else {
                println!("Tags list is not an array for category: {}", category);
            }
        }
    } else {
        println!("Tags dict is not an object!");
    }
    
    println!("Final processed tags count: {}", processed_tags.len());
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
    println!("Starting process_json_data");
    println!("Data: {}", serde_json::to_string_pretty(data)?);

    if let Some(post) = data.get("post") {
        println!("Found post data");
        
        if let Some(file_data) = post.get("file") {
            println!("Found file data");
            
            if let Some(url) = file_data.get("url").and_then(|u| u.as_str()) {
                println!("Found URL: {}", url);
                
                let filename = Path::new(url).file_stem().unwrap().to_str().unwrap();
                let caption_path = file_path.with_file_name(format!("{filename}.txt"));
                println!("Caption path will be: {:?}", caption_path);

                let rating = post
                    .get("rating")
                    .and_then(|r| r.as_str())
                    .unwrap_or("q");
                println!("Rating: {}", rating);
                
                let rating_str = match rating {
                    "s" => "safe, ",
                    "e" => "nsfw, ",
                    _ => "questionable, ",
                };

                let mut caption_content = String::from(rating_str);
                println!("Initial caption content: {}", caption_content);

                if let Some(tags_data) = post.get("tags") {
                    println!("Found tags data: {}", serde_json::to_string_pretty(tags_data)?);
                    
                    let processed_tags = process_tags(tags_data);
                    println!("Processed tags: {:?}", processed_tags);
                    
                    if !processed_tags.is_empty() {
                        caption_content.push_str(&processed_tags.join(", "));
                        
                        println!("{}", "-".repeat(50));
                        println!("Caption file: {caption_path:?}");
                        println!("Tags: {caption_content}");
                        println!("{}", "-".repeat(50));

                        println!("Attempting to write file...");
                        match write_to_file(&caption_path, &caption_content).await {
                            Ok(_) => println!("Successfully wrote file"),
                            Err(e) => println!("Error writing file: {}", e),
                        }
                    } else {
                        println!("No processed tags found!");
                    }
                } else {
                    println!("No tags data found in post!");
                }
            } else {
                println!("No URL found in file data!");
            }
        } else {
            println!("No file data found in post!");
        }
    } else {
        println!("No post data found!");
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
            println!("Starting JSON processing for file");
            match process_json_data(&data_owned, &file_path).await {
                Ok(_) => println!("Successfully processed JSON data"),
                Err(e) => println!("Error processing JSON data: {}", e),
            }
            Ok(())
        }
    }).await.map_err(anyhow::Error::from)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting application");
    
    // Get the target directory from command line args or use current directory
    let root_directory = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::current_dir().expect("Failed to get current directory")
        });
    
    println!("Root directory: {:?}", root_directory);
    
    walk_directory(&root_directory, "json", process_file).await?;
    println!("Finished processing");
    Ok(())
}
