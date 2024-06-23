// This program demonstrates the use of Spherical Linear Interpolation (SLERP) between two quaternions.
//
// The `slerp` function takes two unit quaternions `q0` and `q1`, and a scalar `t` as input.
// It calculates the dot product and the angle between the two quaternions, scales them based on
// the interpolation factor `t`, and returns the interpolated quaternion.
//
// In the `main` function, two unit quaternions `q0` and `q1` are defined. A loop is run where the
// elapsed time is calculated and the interpolation factor `t` is determined. The `slerp` function
// is then called with `q0`, `q1`, and `t` to get the interpolated quaternion, which is printed to
// the console.
//
// The `test_slerp` function in the `tests` module tests the `slerp` function with two specific
// quaternions and an interpolation factor of 0.5. The result is compared with the expected
// output using the `assert_relative_eq!` macro from the `approx` crate.

use nalgebra::UnitQuaternion;
use std::time::{ Duration, Instant };

fn slerp(q0: UnitQuaternion<f32>, q1: UnitQuaternion<f32>, t: f32) -> UnitQuaternion<f32> {
    let dot = q0.quaternion().dot(&q1.quaternion());
    let theta = dot.acos();
    let scale0 = ((1.0 - t) * theta.sin()) / theta.sin();
    let scale1 = (t * theta.sin()) / theta.sin();
    let q = q0.quaternion() * scale0 + q1.quaternion() * scale1;
    UnitQuaternion::from_quaternion(q)
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
        let interpolated = slerp(q0, q1, t);
        println!("Interpolated Quaternion at t={} => {:?}", t, interpolated);
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
