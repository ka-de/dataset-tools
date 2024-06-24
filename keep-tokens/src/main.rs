use std::fs::File;
use std::io::{ BufRead, BufReader, Write };
use std::path::Path;
use walkdir::WalkDir;

/// This code is designed to process a directory of .txt files.
///
/// It defines a list of "keep tokens" that should be retained in the files.
/// For each .txt file found, it reads the content, splits it into tags
/// (separated by commas) and sentences (after the first comma-separated list).
///
/// Finally, it writes a new version of the file with the
/// format: keep_tokens ||| filtered_tags, sentences.
///
/// The code uses the `walkdir` crate to recursively traverse the directory and find
/// the .txt files, which simplifies the code compared to using the standard library's
/// `read_dir` function.

fn main() {
    // Defines a vector of tokens to keep
    let keep_tokens = ["feral", "weasel"];
    // Sets the directory path to search for .txt files
    let directory = Path::new("E:\\training_dir_staging\\1_feral_weasel");
    let mut files = Vec::new(); // Creates an empty vector to store the file paths

    println!("Searching for .txt files in directory: {}", directory.display());

    // Uses the `walkdir` crate to find .txt files recursively
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            !entry.path().is_dir() &&
                entry.path().extension().unwrap_or_default().to_str().unwrap_or_default() ==
                    "txt" &&
                !entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains("-sample-prompts.txt") &&
                entry.path().file_name().unwrap().to_string_lossy() != "sample-prompts.txt"
        }) {
        files.push(entry.path().to_string_lossy().into_owned());
        println!("Found file: {}", entry.path().display());
    }

    println!("Found {} .txt files", files.len());

    for file in &files {
        // Iterates over the found files
        println!("Processing file: {}", file);
        let content = read_file(file); // Reads the content of the file
        let (tags, sentences) = split_content(&content); // Splits the content into tags and sentences

        let filtered_tags: Vec<_> = tags
            .into_iter()
            .filter(|tag| !keep_tokens.contains(tag)) // Filters out tags not in keep_tokens
            .collect();

        let new_content = format!(
            "{} ||| {}, {}",
            keep_tokens.join(", "), // Joins the keep_tokens vector with commas
            filtered_tags.join(","), // Joins the filtered_tags vector with commas
            sentences
        );

        write_file(file, &new_content); // Writes the new content to the file
        println!("Wrote new content to file: {}", file);
    }
}

// Function to read the content of a file
fn read_file(file: &str) -> String {
    println!("Reading file: {}", file);
    let file = File::open(file).unwrap();
    let reader = BufReader::new(file);
    // Reads the lines from the file and collects them into a single string
    reader.lines().collect::<Result<String, _>>().unwrap()
}

// Function to split the content into tags and sentences
fn split_content(content: &str) -> (Vec<&str>, &str) {
    // Splits the content at the "., " pattern
    let split: Vec<_> = content.split("., ").collect();
    // Splits the first part (tags) at commas
    let tags: Vec<_> = split[0].split(',').collect();
    // Gets the second part (sentences) or an empty string if it doesn't exist
    let sentences = split.get(1).unwrap_or(&"");
    // Returns a tuple with the tags vector and trimmed sentences
    (tags, sentences.trim())
}

// Function to write content to a file
fn write_file(file: &str, content: &str) {
    println!("Writing new content to file: {}", file);
    let mut file = File::create(file).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
