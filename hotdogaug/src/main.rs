// hotdogaug/src/main.rs

// This program augments the training images with character segmentation and hue shifting.

// Turn clippy into a real nerd
#![warn(clippy::all, clippy::pedantic)]

use opencv::{ core, imgproc, imgcodecs, prelude::*, photo };
use rand::Rng;
use std::time::Instant;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> opencv::Result<()> {
    let start_time = Instant::now();

    let image_files: Vec<String> = WalkDir::new("test-images")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| { ext == "png" || ext == "jpg" || ext == "jpeg" })
        })
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect();

    for image_file in image_files {
        let image = imgcodecs::imread(&image_file, imgcodecs::IMREAD_COLOR)?;

        let segmented = segment_characters(&image, 50)?;
        imgcodecs::imwrite(
            &format!("segmented_{}", Path::new(&image_file).file_name().unwrap().to_str().unwrap()),
            &segmented,
            &core::Vector::new()
        )?;

        let augmented_image = color_aug(&image, &segmented, 300)?;

        let smoothed_augmented_image = smooth_color_transitions(&augmented_image)?;
        imgcodecs::imwrite(
            &format!(
                "smoothed_augmented_{}",
                Path::new(&image_file).file_name().unwrap().to_str().unwrap()
            ),
            &smoothed_augmented_image,
            &core::Vector::new()
        )?;
    }

    let end_time = Instant::now();
    let execution_time = end_time.duration_since(start_time);
    println!("Executed in {} seconds", execution_time.as_secs_f64());

    Ok(())
}

fn smooth_color_transitions(input_image: &core::Mat) -> opencv::Result<core::Mat> {
    let mut rng = rand::thread_rng();
    let diameter = rng.gen_range(30..46);
    let sigma_color = rng.gen_range(40..61);
    let sigma_space = rng.gen_range(80..101);
    let mut output = core::Mat::default();
    imgproc::bilateral_filter(
        input_image,
        &mut output,
        diameter,
        sigma_color as f64,
        sigma_space as f64,
        core::BORDER_DEFAULT
    )?;
    Ok(output)
}

fn segment_characters(image: &core::Mat, padding: i32) -> opencv::Result<core::Mat> {
    let mut denoised = core::Mat::default();
    photo::fast_nl_means_denoising_colored(image, &mut denoised, 10.0, 10.0, 7, 21)?;

    let mut image_padded = core::Mat::default();
    core::copy_make_border(
        &denoised,
        &mut image_padded,
        padding,
        padding,
        padding,
        padding,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0)
    )?;

    let mut gray = core::Mat::default();
    imgproc::cvt_color(&image_padded, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

    let mut blurred = core::Mat::default();
    imgproc::gaussian_blur(
        &gray,
        &mut blurred,
        core::Size::new(9, 9),
        15.0,
        15.0,
        core::BORDER_DEFAULT
    )?;

    let mut median_blurred = core::Mat::default();
    imgproc::median_blur(&blurred, &mut median_blurred, 7)?;

    let mut thresh = core::Mat::default();
    imgproc::threshold(
        &median_blurred,
        &mut thresh,
        0.0,
        255.0,
        imgproc::THRESH_BINARY_INV | imgproc::THRESH_OTSU
    )?;

    let mut contours = core::Vector::<core::Vector<core::Point>>::new();
    imgproc::find_contours(
        &thresh,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0)
    )?;

    let mut rng = rand::thread_rng();
    let mut output = core::Mat::zeros(thresh.rows(), thresh.cols(), thresh.typ())?.to_mat()?;

    for contour in contours.iter() {
        let epsilon =
            (if rng.gen_bool(0.01) { 0.5 } else { rng.gen_range(0.3..0.35) }) *
            imgproc::arc_length(&contour, true)?;
        let mut approx = core::Vector::<core::Point>::new();
        imgproc::approx_poly_dp(&contour, &mut approx, epsilon, true)?;

        let rect = imgproc::bounding_rect(&approx)?;
        let roi = thresh.roi(rect)?;
        let mut roi_output = output.roi_mut(rect)?;
        core::bitwise_and(&roi, &roi, &mut roi_output, &core::no_array())?;
    }

    // Draw contours
    imgproc::draw_contours(
        &mut output,
        &contours,
        -1, // draw all contours
        core::Scalar::new(255.0, 255.0, 255.0, 0.0), // white color
        1, // thickness
        imgproc::LINE_8,
        &core::no_array(),
        i32::MAX,
        core::Point::new(0, 0)
    )?;

    let mut output_no_padding = core::Mat::default();
    let roi = core::Rect::new(padding, padding, image.cols(), image.rows());
    output.roi(roi)?.copy_to(&mut output_no_padding)?;

    let mut guide = core::Mat::default();
    imgproc::cvt_color(image, &mut guide, imgproc::COLOR_BGR2GRAY, 0)?;

    // Simplified filtering as a replacement for guided_filter
    let mut filtered = core::Mat::default();
    imgproc::gaussian_blur(
        &output_no_padding,
        &mut filtered,
        core::Size::new(5, 5),
        0.0,
        0.0,
        core::BORDER_DEFAULT
    )?;

    Ok(filtered)
}

fn color_aug(
    input_image: &core::Mat,
    mask: &core::Mat,
    value_threshold: i32
) -> opencv::Result<core::Mat> {
    let mut hsv = core::Mat::default();
    imgproc::cvt_color(input_image, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let mut rng = rand::thread_rng();
    let random_hue = rng.gen_range(0..180);

    let mut mask_bright = core::Mat::default();
    let mut hsv_channels = core::Vector::<core::Mat>::new();
    core::split(&hsv, &mut hsv_channels)?;
    core::compare(
        &hsv_channels.get(2)?,
        &core::Scalar::all(value_threshold as f64),
        &mut mask_bright,
        core::CMP_GT
    )?;

    let mut combined_mask = core::Mat::default();
    core::bitwise_and(mask, &mask_bright, &mut combined_mask, &core::no_array())?;

    let hue_channel = hsv_channels.get(0)?.clone();
    let mut hue_channel_clone = core::Mat::default();
    core::add(
        &hue_channel,
        &core::Scalar::all(random_hue as f64),
        &mut hue_channel_clone,
        &combined_mask,
        -1
    )?;
    let mut subtracted_hue_channel = core::Mat::default();
    core::subtract(
        &hue_channel_clone,
        &core::Scalar::all(180.0),
        &mut subtracted_hue_channel,
        &core::no_array(),
        -1
    )?;
    hsv_channels.set(0, subtracted_hue_channel)?;

    core::merge(&hsv_channels, &mut hsv)?;

    let mut output = core::Mat::default();
    imgproc::cvt_color(&hsv, &mut output, imgproc::COLOR_HSV2BGR, 0)?;

    Ok(output)
}
