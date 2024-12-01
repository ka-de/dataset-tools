use std::path::Path;
use anyhow::Result;
use dataset_tools::{walk_directory, read_file_content, write_to_file};

#[tokio::main]
async fn main() -> Result<()> {
    // Get directory from args or use current directory
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    walk_directory(Path::new(&directory), "txt", |path| async move {
        // Skip specific files
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.contains("wordfreq.txt") || file_name.contains("sample-prompts.txt") {
                return Ok(());
            }
        }

        // Read file content
        let content = read_file_content(path.to_str().unwrap()).await?;
        
        // Process the content
        let fixed_content = fix_tags(&content);
        
        // Write back to file
        write_to_file(&path, &fixed_content).await?;
        
        println!("Processed: {}", path.display());
        Ok(())
    })
    .await?;

    Ok(())
}

fn fix_tags(content: &str) -> String {
    // Split content into lines and process each line
    let tags: Vec<String> = content
        .lines()
        .flat_map(|line| line.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    // Join all tags with comma and space
    tags.join(", ")
}
