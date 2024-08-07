use std::path::{ Path, PathBuf };
use tokio::fs;
use anyhow::{ Context, Result };
use md5::{ Md5, Digest };
use std::ffi::OsStr;
use std::env;
use std::pin::Pin;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <target_directory>", args[0]);
        std::process::exit(1);
    }

    let target_dir = Path::new(&args[1]);
    if !target_dir.is_dir() {
        eprintln!("Error: The specified path is not a directory.");
        std::process::exit(1);
    }

    println!("Processing directory: {}", target_dir.display());

    process_directory(target_dir).await?;

    println!("Finished processing all files.");
    Ok(())
}

async fn process_directory(dir: &Path) -> Result<()> {
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            // Box the recursive call
            Pin::from(Box::new(process_directory(&path))).await?;
        } else {
            process_file(path).await?;
        }
    }
    Ok(())
}

async fn process_file(path: PathBuf) -> Result<()> {
    println!("Examining file: {}", path.display());

    if is_image_file(&path) {
        println!("File is an image: {}", path.display());
        let md5_sum = calculate_md5(&path).await?;
        println!("Calculated MD5: {}", md5_sum);

        let new_name = format!(
            "{}.{}",
            md5_sum,
            path.extension().unwrap_or_default().to_str().unwrap_or("")
        );
        let new_path = path.with_file_name(&new_name);

        println!("Attempting to rename: {} -> {}", path.display(), new_path.display());

        match fs::rename(&path, &new_path).await {
            Ok(_) => println!("Successfully renamed: {} -> {}", path.display(), new_path.display()),
            Err(e) => println!("Failed to rename {}: {}", path.display(), e),
        }

        // Check for associated text file
        let txt_path = path.with_extension("txt");
        if txt_path.exists() {
            let new_txt_name = format!("{}.txt", md5_sum);
            let new_txt_path = txt_path.with_file_name(&new_txt_name);
            match fs::rename(&txt_path, &new_txt_path).await {
                Ok(_) =>
                    println!(
                        "Renamed associated text file: {} -> {}",
                        txt_path.display(),
                        new_txt_path.display()
                    ),
                Err(e) => println!("Failed to rename text file {}: {}", txt_path.display(), e),
            }
        }
    } else {
        println!("File is not an image: {}", path.display());
    }

    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "webp" | "jpg" | "jpeg" | "jxl" | "png"),
        None => false,
    }
}

async fn calculate_md5(path: &Path) -> Result<String> {
    let contents = fs::read(path).await.context("Failed to read file")?;
    let mut hasher = Md5::new();
    hasher.update(&contents);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
