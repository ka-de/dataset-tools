// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use dataset_tools_rs::{ walk_directory, write_to_file };
use std::path::Path;

fn create_caption_file(path: &Path) -> std::io::Result<()> {
    let caption_file = path.with_extension("txt");
    if !caption_file.exists() {
        write_to_file(&caption_file, "")?;
        println!("Created caption file: {}", caption_file.display());
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let directory = Path::new("E:\\training_dir_staging");
    walk_directory(directory, "jpg", create_caption_file)?;
    walk_directory(directory, "jpeg", create_caption_file)?;
    walk_directory(directory, "png", create_caption_file)?;
    println!("All caption files have been created.");
    Ok(())
}
