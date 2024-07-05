// search-for-empty-captions\src\main.rs

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, is_image_file, caption_file_exists_and_not_empty };
use std::{ path::Path, sync::Arc };
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root_dir = Path::new(r"E:\training_dir\_");
    let missing_captions = Arc::new(Mutex::new(Vec::new()));

    walk_directory(root_dir, "", |path| {
        let missing_captions = Arc::clone(&missing_captions);
        async move {
            if is_image_file(&path) {
                let caption_path = path.with_extension("txt");
                if !caption_file_exists_and_not_empty(&caption_path).await {
                    missing_captions.lock().await.push(path.to_string_lossy().to_string());
                }
            }
            Ok(())
        }
    }).await?;

    let missing_captions = missing_captions.lock().await;

    if missing_captions.is_empty() {
        println!("All image files have corresponding non-empty caption files.");
    } else {
        println!(
            "The following image files are missing caption files or have empty caption files:"
        );
        for path in missing_captions.iter() {
            println!("{path}");
        }
    }

    Ok(())
}
