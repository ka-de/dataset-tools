// In this modified version, the interpolation factor t is passed through the ease_out_cubic
// function before it’s used in the slerp function. This easing function will cause the
// interpolation to start quickly and then slow down towards the end, giving a more natural
// feel to the animation.
//
// You can replace ease_out_cubic with any other easing function to achieve different effects

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

fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
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
        let eased_t = ease_out_cubic(t); // apply easing function
        let interpolated = slerp(q0, q1, eased_t);
        println!("Interpolated Quaternion at t={} => {:?}", eased_t, interpolated);
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
