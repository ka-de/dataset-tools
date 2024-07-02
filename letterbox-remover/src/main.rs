extern crate image;

use image::{ GenericImageView, GenericImage, ImageBuffer, Rgba };
use std::path::Path;
use std::io::{ self, ErrorKind };

fn is_black(pixel: &Rgba<u8>) -> bool {
    pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0
}

fn main() -> Result<(), image::ImageError> {
    let mut img = image
        ::open(&Path::new("input.png"))
        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
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
    cropped.save("output.png")?;

    Ok(())
}
