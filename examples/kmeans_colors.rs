use image::{ ImageBuffer, Rgba };
use palette::{ IntoColor, Lab, Srgb };
use kmeans_colors::get_kmeans;

fn main() {
    let img = image::open("assets/input.png").unwrap().to_rgba8();
    let (width, height) = img.dimensions();

    let pixels: Vec<Rgba<u8>> = img
        .into_raw()
        .chunks(4)
        .map(|chunk| { Rgba([chunk[0], chunk[1], chunk[2], chunk[3]]) })
        .collect();
    let num_colors = 6; // specify the number of colors to reduce to

    let lab: Vec<Lab> = pixels
        .iter()
        .map(|x| {
            let srgb = Srgb::new(
                (x.0[0] as f32) / 255.0,
                (x.0[1] as f32) / 255.0,
                (x.0[2] as f32) / 255.0
            );
            srgb.into_format().into_color()
        })
        .collect();

    let kmeans_result = get_kmeans(
        num_colors,
        20, // max iterations
        5.0, // convergence threshold
        true, // verbose
        &lab,
        0 // seed
    );

    let output_pixels: Vec<u8> = lab
        .iter()
        .zip(pixels.iter())
        .map(|(lab_color, original_pixel)| {
            let closest_centroid_index = kmeans_result.centroids
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let distance_a = color_difference(lab_color, a);
                    let distance_b = color_difference(lab_color, b);
                    distance_a.partial_cmp(&distance_b).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(index, _)| index)
                .unwrap();

            let closest_centroid = &kmeans_result.centroids[closest_centroid_index];
            let srgb: Srgb = Lab::new(
                closest_centroid.l,
                closest_centroid.a,
                closest_centroid.b
            ).into_color();
            let r = (srgb.red * 255.0) as u8;
            let g = (srgb.green * 255.0) as u8;
            let b = (srgb.blue * 255.0) as u8;
            [r, g, b, original_pixel.0[3]]
        })
        .flatten()
        .collect();

    let output_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, output_pixels).unwrap();
    output_img.save("output.png").unwrap();
}

fn color_difference(a: &Lab, b: &Lab) -> f32 {
    let dl = a.l - b.l;
    let da = a.a - b.a;
    let db = a.b - b.b;
    (dl * dl + da * da + db * db).sqrt()
}
