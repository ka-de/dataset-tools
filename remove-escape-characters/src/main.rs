use anyhow::Result;
use dataset_tools::{ walk_directory, read_file_content, write_to_file };
use std::path::Path;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <target_directory>", args[0]);
        std::process::exit(1);
    }

    let target_dir = Path::new(&args[1]);

    if !target_dir.is_dir() {
        eprintln!("Error: {} is not a valid directory", target_dir.display());
        std::process::exit(1);
    }

    println!("Processing .txt files in: {}", target_dir.display());

    walk_directory(target_dir, "txt", process_txt_file).await?;

    println!("Finished processing all .txt files!");
    Ok(())
}

async fn process_txt_file(path: std::path::PathBuf) -> Result<()> {
    println!("Processing file: {}", path.display());

    // Read the content of the file
    let content = match path.to_str() {
        Some(str_path) => read_file_content(str_path).await?,
        None => {
            return Err(anyhow::anyhow!("Invalid path"));
        }
    };

    // Remove all backslash characters
    let processed_content = content.replace('\\', "");

    // Write the processed content back to the file
    write_to_file(&path, &processed_content).await?;

    println!("Processed file: {}", path.display());
    Ok(())
}
