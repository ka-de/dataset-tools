// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools_rs::{ walk_directory, is_image_file, caption_file_exists_and_not_empty };
use std::path::Path;

fn main() -> std::io::Result<()> {
    let root_dir = Path::new(r"E:\training_dir\_");
    let mut missing_captions = Vec::new();

    walk_directory(root_dir, "", |path| {
        if is_image_file(path) {
            let caption_path = path.with_extension("txt");
            if !caption_file_exists_and_not_empty(&caption_path) {
                missing_captions.push(path.to_string_lossy().to_string());
            }
        }
        Ok(())
    })?;

    if missing_captions.is_empty() {
        println!("All image files have corresponding non-empty caption files.");
    } else {
        println!(
            "The following image files are missing caption files or have empty caption files:"
        );
        for path in missing_captions {
            println!("{}", path);
        }
    }

    Ok(())
}
