// remove-transparency\src\main.rs

// This program removes the alpha channel from PNG files in a target directory and subdirectories.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use std::path::PathBuf;
use image::{ GenericImageView, ImageBuffer, Rgba };
use dataset_tools::{ walk_directory, is_image_file };
use anyhow::{ Context, Result };
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target directory to process PNG files
    #[arg(short, long)]
    target: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let target_dir = args.target.unwrap_or_else(|| PathBuf::from("."));
    println!("Processing PNG files in: {:?}", target_dir);

    walk_directory(&target_dir, "png", process_image).await?;

    println!("All PNG files processed successfully.");
    Ok(())
}

async fn process_image(path: PathBuf) -> Result<()> {
    if !is_image_file(&path) {
        return Ok(());
    }

    println!("Processing image: {:?}", path);

    let img = image::open(&path).context("Failed to open image")?;
    let (width, height) = img.dimensions();

    let mut new_image = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.pixels() {
        let new_pixel = if pixel[3] == 0 {
            Rgba([0, 0, 0, 255]) // Black, fully opaque
        } else {
            pixel
        };
        new_image.put_pixel(x, y, new_pixel);
    }

    new_image.save(&path).context("Failed to save image")?;
    println!("Processed and saved: {:?}", path);

    Ok(())
}
