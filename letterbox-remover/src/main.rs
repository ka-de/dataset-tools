extern crate image;
extern crate walkdir;

use std::fs;
use std::path::{ Path, PathBuf };
use std::io::{ self, ErrorKind };

use image::{ GenericImageView, Rgba };
use walkdir::WalkDir;

fn is_black(pixel: &Rgba<u8>) -> bool {
    pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0
}

fn process_image(input_path: &Path) -> Result<(), image::ImageError> {
    let mut img = image::open(input_path)?;
    let (width, height) = img.dimensions();

    let mut top = 0;
    let mut bottom = height;
    let mut left = 0;
    let mut right = width;

    'outer: for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if !is_black(&pixel) {
                top = y;
                break 'outer;
            }
        }
    }

    'outer: for y in (0..height).rev() {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            if !is_black(&pixel) {
                bottom = y;
                break 'outer;
            }
        }
    }

    'outer: for x in 0..width {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            if !is_black(&pixel) {
                left = x;
                break 'outer;
            }
        }
    }

    'outer: for x in (0..width).rev() {
        for y in 0..height {
            let pixel = img.get_pixel(x, y);
            if !is_black(&pixel) {
                right = x;
                break 'outer;
            }
        }
    }

    let cropped = img.crop(left, top, right - left, bottom - top);

    let output_path = generate_output_path(input_path)?;
    cropped.save(output_path)?;

    Ok(())
}

fn generate_output_path(input_path: &Path) -> io::Result<PathBuf> {
    let stem = input_path
        .file_stem()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid file stem"))?
        .to_str()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid unicode in file stem"))?;
    let extension = input_path
        .extension()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid file extension"))?
        .to_str()
        .ok_or_else(||
            io::Error::new(ErrorKind::InvalidInput, "Invalid unicode in file extension")
        )?;
    let parent = input_path
        .parent()
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Invalid parent directory"))?;
    Ok(parent.join(format!("{}-cropped.{}", stem, extension)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let delete_original = args.contains(&"-d".to_string());
    let input_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };
    if input_path.is_dir() {
        for entry in WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if let Some(ext) = extension.to_str() {
                        if ["png", "jpg", "jpeg"].contains(&ext.to_lowercase().as_str()) {
                            match process_image(path) {
                                Ok(_) => {
                                    println!("Processed: {:?}", path);
                                    if delete_original {
                                        fs::remove_file(path)?;
                                    }
                                }
                                Err(e) => eprintln!("Error processing {:?}: {}", path, e),
                            }
                        }
                    }
                }
            }
        }
    } else if input_path.is_file() {
        process_image(&input_path)?;
        println!("Processed: {:?}", input_path);
        if delete_original {
            fs::remove_file(input_path)?;
        }
    } else {
        return Err(Box::new(io::Error::new(ErrorKind::NotFound, "Input path not found")));
    }
    Ok(())
}
