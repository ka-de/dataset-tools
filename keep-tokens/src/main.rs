// keep-tokens\src\main.rs

// This code is designed to process a directory of .txt files.
//
// It defines a list of "keep tokens" that should be retained in the files.
// For each .txt file found, it reads the content, splits it into tags
// (separated by commas) and sentences (after the first comma-separated list).
//
// Finally, it writes a new version of the file with the
// format: `keep_tokens ||| filtered_tags, sentences`.
//
// The code uses the `walkdir` crate to recursively traverse the directory and find
// the .txt files, which simplifies the code compared to using the standard library's
// `read_dir` function.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, read_file_content, split_content, write_to_file };
use std::path::PathBuf;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let keep_tokens = ["feral", "weasel"];
    let directory = PathBuf::from("E:\\training_dir_staging");

    println!("Searching for .txt files in directory: {}", directory.display());

    walk_directory(&directory, "txt", |path| {
        let path_buf = path.to_path_buf();
        async move {
            if
                !path_buf.file_name().unwrap().to_string_lossy().contains("-sample-prompts.txt") &&
                path_buf.file_name().unwrap().to_string_lossy() != "sample-prompts.txt"
            {
                println!("Processing file: {}", path_buf.display());
                let content = read_file_content(path_buf.to_str().unwrap()).await?;
                let (tags, sentences) = split_content(&content);

                let filtered_tags: Vec<_> = tags
                    .into_iter()
                    .filter(|tag| !keep_tokens.contains(tag))
                    .collect();

                let new_content = format!(
                    "{} ||| {}, {}",
                    keep_tokens.join(", "),
                    filtered_tags.join(","),
                    sentences
                );

                write_to_file(&path_buf, &new_content).await?;
                println!("Wrote new content to file: {}", path_buf.display());
            }
            Ok(())
        }
    }).await?;

    Ok(())
}
