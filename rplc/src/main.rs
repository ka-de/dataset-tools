use std::env;
use std::path::Path;
use anyhow::Result;
use dataset_tools::walk_directory;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: ./rplc.exe <search_string> <replace_string>");
        std::process::exit(1);
    }

    let search_string = args[1].clone();
    let replace_string = args[2].clone();

    println!("Replacing '{search_string}' with '{replace_string}' in all .txt files...");

    let search_string_clone = search_string.clone();
    let replace_string_clone = replace_string.clone();

    walk_directory(".", "txt", move |path| {
        let search = search_string_clone.clone();
        let replace = replace_string_clone.clone();
        async move { process_file(&path, &search, &replace).await }
    }).await?;

    println!("Replacement complete.");
    Ok(())
}

async fn process_file(path: &Path, search: &str, replace: &str) -> Result<()> {
    let content = fs::read_to_string(path).await?;
    let new_content = content.replace(search, replace);

    if content != new_content {
        fs::write(path, new_content).await?;
        println!("Updated: {}", path.display());
    }

    Ok(())
}
