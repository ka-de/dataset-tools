use rand::Rng;
use std::sync::{ Arc, Mutex };
use std::thread;
use std::time::Duration;

// Define a struct to represent a resource manager
#[derive(PartialEq)]
struct ResourceManager {
    wood: u32,
    stone: u32,
    food: u32,
}

// Define an enum to represent different types of resources
enum Resource {
    Wood,
    Stone,
    Food,
}

impl ResourceManager {
    // Gather a resource (wood, stone, or food) and return the amount gathered and the remaining amount
    fn gather_resource(&mut self, resource: &Resource) -> (u32, u32) {
        // Determine which resource to gather based on the input resource type
        let resource_amount = match resource {
            Resource::Wood => &mut self.wood,
            Resource::Stone => &mut self.stone,
            Resource::Food => &mut self.food,
        };

        // If there is enough of the resource available, gather a random amount
        if *resource_amount > 0 {
            let amount = rand::thread_rng().gen_range(1..=*resource_amount);
            *resource_amount -= amount;
            (amount, *resource_amount)
        } else {
            // If there is not enough of the resource available, return 0 and the remaining amount
            (0, *resource_amount)
        }
    }
}

// Define a struct to represent a player
struct Player {
    id: u32,
    resource_manager: Arc<Mutex<ResourceManager>>,
    gathered_resources: Mutex<u32>,
}

// Implement methods for the Player struct
impl Player {
    // Create a new player with a given ID and resource manager
    fn new(id: u32, resource_manager: Arc<Mutex<ResourceManager>>) -> Self {
        Player {
            id,
            resource_manager,
            gathered_resources: Mutex::new(0),
        }
    }

    // Gather a resource (wood, stone, or food) and update the player's gathered resources
    fn gather_resource(&self, resource: &Resource) {
        // Lock the resource manager and gathered resources to ensure thread safety
        let mut resource_manager = self.resource_manager.lock().unwrap();
        let mut gathered_resources = self.gathered_resources.lock().unwrap();

        // Gather the resource using the resource manager
        let (amount, remaining) = resource_manager.gather_resource(resource);

        // Determine the resource name based on the input resource type
        let resource_name = match resource {
            Resource::Wood => "wood",
            Resource::Stone => "stone",
            Resource::Food => "food",
        };

        if amount > 0 {
            *gathered_resources += amount;
            println!(
                "Player {} gathered {} {}. Remaining {}: {}",
                self.id,
                amount,
                resource_name,
                resource_name,
                remaining
            );
        } else {
            println!(
                "Player {} failed to gather {}. Remaining {}: {}",
                self.id,
                resource_name,
                resource_name,
                remaining
            );
        }
    }
}

fn main() {
    // Create a resource manager with initial resources
    let resource_manager = Arc::new(
        Mutex::new(ResourceManager {
            wood: 10,
            stone: 50,
            food: 20,
        })
    );

    // Create a vector to store player handles and a vector to store players
    let mut player_handles = vec![];
    let mut players = vec![];

    // Create 5 players and start a thread for each player
    for i in 0..5 {
        // Clone the resource manager for each player
        let resource_manager = Arc::clone(&resource_manager);
        // Create a new player and add it to the players vector
        let player = Arc::new(Player::new(i, resource_manager));
        players.push(Arc::clone(&player));
        // Start a thread for the player
        let handle = thread::spawn(move || {
            // Create an empty resource manager to check for when resources are depleted
            let empty_resources = ResourceManager {
                wood: 0,
                stone: 0,
                food: 0,
            };
            // Loop until all resources are depleted
            while *player.resource_manager.lock().unwrap() != empty_resources {
                // Randomly select a resource to gather
                let resource = match rand::thread_rng().gen_range(0..3) {
                    0 => Resource::Wood,
                    1 => Resource::Stone,
                    2 => Resource::Food,
                    _ => unreachable!(),
                };
                // Gather the resource
                player.gather_resource(&resource); // Pass a reference to resource
                // Sleep for a random amount of time before gathering again
                thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(1..5)));
            }
        });

        // Add the thread handle to the player handles vector
        player_handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in player_handles {
        handle.join().unwrap();
    }

    // Determine the winner by finding the player with the most gathered resources
    let mut max_resources = 0;
    let mut winner_id = 0;

    for player in &players {
        let resources = *player.gathered_resources.lock().unwrap();
        if resources > max_resources {
            max_resources = resources;
            winner_id = player.id;
        }
    }

    // Print the winner
    println!("Player {} wins with {} resources!", winner_id, max_resources);
}
