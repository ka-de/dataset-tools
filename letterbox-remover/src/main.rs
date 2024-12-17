// letterbox-remover/src/main.rs

// This program removes letterboxes from images in a target directory and subdirectories.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use image::{DynamicImage, ImageDecoder};
use jxl_oxide::integration::JxlDecoder;
use std::env;
use std::fs::File;
use std::path::Path;
use walkdir::WalkDir;
use dataset_tools::remove_letterbox;

async fn process_jxl(path: &Path) -> std::io::Result<()> {
    let file = File::open(path)?;
    let decoder = JxlDecoder::new(file).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    // Convert to DynamicImage
    let image = DynamicImage::from_decoder(decoder)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    // Process the image to remove letterbox
    // ... (implement letterbox removal logic here)
    
    // Save back as PNG (or original format if needed)
    let output_path = path.with_extension("png");
    let output_file = File::create(output_path)?;
    let encoder = image::codecs::png::PngEncoder::new(output_file);
    
    image.write_with_encoder(encoder)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

fn is_jxl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext.eq_ignore_ascii_case("jxl"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_directory>", args[0]);
        std::process::exit(1);
    }

    let input_dir = Path::new(&args[1]);

    for entry in WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if is_jxl_file(path) {
                println!("Processing JXL: {}", path.display());
                process_jxl(path).await?;
            } else if dataset_tools::is_image_file(path) {
                println!("Processing: {}", path.display());
                remove_letterbox(path).await?;
            }
        }
    }

    Ok(())
}
