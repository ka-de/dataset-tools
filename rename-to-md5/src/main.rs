use std::path::{ Path, PathBuf };
use std::collections::HashMap;
use tokio::fs;
use anyhow::{ Context, Result, anyhow };
use md5::{ Md5, Digest };
use dataset_tools::{ walk_directory, is_image_file };
use std::env;

async fn process_image(path: PathBuf) -> Result<()> {
    let content = fs::read(&path).await.context("Failed to read file")?;
    let md5_sum = format!("{:x}", Md5::digest(&content));

    let parent = path.parent().unwrap_or(Path::new(""));
    let new_path = parent.join(
        format!("{}.{}", md5_sum, path.extension().unwrap().to_str().unwrap())
    );

    // Rename the image file
    fs::rename(&path, &new_path).await.context("Failed to rename file")?;

    // Rename associated caption files
    let caption_extensions = [".txt", ".caption"];
    for ext in &caption_extensions {
        let caption_path = path.with_extension(ext.trim_start_matches('.'));
        if caption_path.exists() {
            let new_caption_path = new_path.with_extension(ext.trim_start_matches('.'));
            fs
                ::rename(&caption_path, &new_caption_path).await
                .context("Failed to rename caption file")?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!("Usage: {} <folder_path>", args[0]));
    }

    let input_dir = PathBuf::from(&args[1]);
    if !input_dir.is_dir() {
        return Err(anyhow!("The provided path is not a directory"));
    }

    let mut md5_map: HashMap<String, PathBuf> = HashMap::new();

    let process_file = |path: PathBuf| async move {
        if is_image_file(&path) {
            let content = fs::read(&path).await.context("Failed to read file")?;
            let md5_sum = format!("{:x}", Md5::digest(&content));

            if let Some(existing_path) = md5_map.get(&md5_sum) {
                // Duplicate found, remove the new file
                fs::remove_file(&path).await.context("Failed to remove duplicate file")?;
                println!("Removed duplicate: {}", path.display());
            } else {
                // Process the new file
                process_image(path.clone()).await?;
                md5_map.insert(md5_sum, path);
            }
        }
        Ok(())
    };

    let extensions = ["jpg", "jpeg", "png", "jxl", "webp"];
    for ext in extensions.iter() {
        walk_directory(&input_dir, ext, process_file).await?;
    }

    println!("Image renaming process completed successfully.");
    Ok(())
}
