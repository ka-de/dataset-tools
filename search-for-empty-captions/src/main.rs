// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools::{ walk_directory, is_image_file, caption_file_exists_and_not_empty };
use std::path::Path;
use tokio::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let root_dir = Path::new(r"E:\training_dir\_");
    let missing_captions = Vec::new();

    walk_directory(root_dir, "", |path| {
        let mut value = missing_captions.clone();
        async move {
            if is_image_file(&path) {
                let caption_path = path.with_extension("txt");
                if !caption_file_exists_and_not_empty(&caption_path).await {
                    value.push(path.to_string_lossy().to_string());
                }
            }
            Ok(())
        }
    }).await?;

    if missing_captions.is_empty() {
        println!("All image files have corresponding non-empty caption files.");
    } else {
        println!(
            "The following image files are missing caption files or have empty caption files:"
        );
        for path in missing_captions {
            println!("{path}");
        }
    }

    Ok(())
}
