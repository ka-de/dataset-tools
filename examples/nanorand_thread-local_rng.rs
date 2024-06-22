
/**
 * Generating a number with a thread-local RNG
 */

use nanorand::Rng;

fn main() {
    let mut rng = nanorand::tls_rng();
    println!("Random number: {}", rng.generate::<u64>());
}
