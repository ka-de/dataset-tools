// In this modified version, the interpolation factor t is doubled when it’s less than 0.5,
// effectively speeding up the interpolation during the first half of the duration.
//
// Once t reaches 0.5, it is clamped to 1.0, which means the interpolation will jump to the
// final value and stay there for the second half of the duration. This approach removes the
// slow down at the end but results in a sudden jump to the final value at the midpoint.
//
// If you want a smoother transition without a sudden jump, consider using an easing function
// that provides a more gradual change in speed. There are many easing functions available that
// can give you the desired effect, such as quadratic, cubic, or sinusoidal easing.

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
        let clamped_t = if t < 0.5 { t * 2.0 } else { 1.0 }; // clamp t to maintain speed
        let interpolated = slerp(q0, q1, clamped_t);
        println!("Interpolated Quaternion at t={} => {:?}", clamped_t, interpolated);
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
