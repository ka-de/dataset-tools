/// * This module provides a `Collectible` trait and two structs `Coin` and `PowerUp` that implement this trait.
///  *
///  * The `Collectible` trait defines a single method `collect`, which does not return a value.
///  *
///  * The `Coin` struct represents a coin with a certain value. When a `Coin` is collected, it prints out a message indicating its value.
///  *
///  * The `PowerUp` struct represents a power-up of a certain kind. When a `PowerUp` is collected, it prints out a message indicating its kind.
///  *
///  * The `collect_item` function takes an item that implements the `Collectible` trait and calls the `collect` method on it.
///  *
///  * In the `main` function, a `Coin` and a `PowerUp` are created and passed to the `collect_item` function.

pub trait Collectible {
    fn collect(&self);
}

pub struct Coin {
    pub value: u32,
}

impl Collectible for Coin {
    fn collect(&self) {
        println!("Collected a coin worth {} points", self.value);
    }
}

pub struct PowerUp {
    pub kind: String,
}

impl Collectible for PowerUp {
    fn collect(&self) {
        println!("Collected a power-up of type: {}", self.kind);
    }
}

fn collect_item<T: Collectible>(item: T) {
    item.collect();
}

fn main() {
    let coin = Coin { value: 10 };
    let power_up = PowerUp { kind: String::from("Invincibility") };

    collect_item(coin); // Monomorphized to collect_item::<Coin>
    collect_item(power_up); // Monomorphized to collect_item::<PowerUp>
}
