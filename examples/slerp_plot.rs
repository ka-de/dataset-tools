// This program demonstrates the Spherical Linear intERPolation (SLERP) between two quaternions.
//
// The `slerp` function takes two unit quaternions and a scalar `t` in the range [0, 1] and returns a unit quaternion
// that represents an interpolated rotation between the two input quaternions.
//
// The `plot_slerp` function takes two unit quaternions and a total duration for the interpolation. It generates a plot
// of the angle between the starting quaternion and the interpolated quaternion over time.
//
// The `main` function creates two unit quaternions, performs the SLERP interpolation between them over a specified
// duration, and prints the interpolated quaternion at each step. After the interpolation, it calls `plot_slerp` to
// generate a plot of the interpolation.
//
// The `test_slerp` function in the `tests` module tests the `slerp` function with specific input and expected output.
//
// # Dependencies
// - nalgebra: for quaternion and other mathematical operations
// - std::time: for time measurement
// - plotters: for plotting the interpolation result
//
// # Panics
// The `plot_slerp` function will panic if it fails to create a drawing area or a chart.
//
// # Errors
// The `plot_slerp` function will return an error if it fails to draw the mesh or the series on the chart.

use nalgebra::UnitQuaternion;
use std::time::{ Duration, Instant };
use plotters::prelude::*;

fn slerp(q0: UnitQuaternion<f32>, q1: UnitQuaternion<f32>, t: f32) -> UnitQuaternion<f32> {
    let dot = q0.quaternion().dot(&q1.quaternion());
    let theta = dot.acos();
    let scale0 = ((1.0 - t) * theta.sin()) / theta.sin();
    let scale1 = (t * theta.sin()) / theta.sin();
    let q = q0.quaternion() * scale0 + q1.quaternion() * scale1;
    UnitQuaternion::from_quaternion(q)
}

fn plot_slerp(
    q0: UnitQuaternion<f32>,
    q1: UnitQuaternion<f32>,
    total_duration: Duration
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("slerp.png", (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("SLERP Interpolation", ("sans-serif", 16).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..1f32, 0f32..1f32)?; // Adjust the range here

    chart.configure_mesh().draw()?;

    let mut data = Vec::new();
    let start_time = Instant::now();
    while start_time.elapsed() < total_duration {
        let t = start_time.elapsed().as_secs_f32() / total_duration.as_secs_f32();
        let clamped_t = if t < 0.5 { t * 2.0 } else { 1.0 };
        let interpolated = slerp(q0, q1, clamped_t);
        let dot = q0.quaternion().dot(&interpolated.quaternion());
        let angle = dot.acos();
        data.push((t, angle));
    }

    chart.draw_series(LineSeries::new(data, &RED))?;

    Ok(())
}

fn main() {
    let q0 = UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0);
    let q1 = UnitQuaternion::from_euler_angles(1.0, 1.0, 1.0);
    let start_time = Instant::now();
    let total_duration = Duration::from_secs(10); // total duration of interpolation

    loop {
        let elapsed_time = start_time.elapsed();
        if elapsed_time > total_duration {
            break;
        }
        let t = elapsed_time.as_secs_f32() / total_duration.as_secs_f32(); // calculate interpolation factor
        let clamped_t = if t < 0.5 { t * 2.0 } else { 1.0 }; // clamp t to maintain speed
        let interpolated = slerp(q0, q1, clamped_t);
        println!("Interpolated Quaternion at t={} => {:?}", clamped_t, interpolated);
    }

    // Call the plot function after the interpolation loop
    if let Err(e) = plot_slerp(q0, q1, total_duration) {
        eprintln!("Error plotting SLERP: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Quaternion;
    use approx::assert_relative_eq;

    #[test]
    fn test_slerp() {
        let q0 = UnitQuaternion::from_quaternion(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        let q1 = UnitQuaternion::from_quaternion(Quaternion::new(0.0, 1.0, 0.0, 0.0));
        let t = 0.5;
        let result = slerp(q0, q1, t);
        let expected = UnitQuaternion::from_quaternion(
            Quaternion::new(0.7071067811865476, 0.7071067811865476, 0.0, 0.0)
        );
        assert_relative_eq!(result, expected, epsilon = 1.0e-6);
    }
}
