
/**
 * Generating a number with an initialized RNG
 */

use nanorand::{ Rng, WyRand };

fn main() {
    let mut rng = WyRand::new();
    println!("Random number: {}", rng.generate::<u64>());
}
