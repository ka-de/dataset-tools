use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

impl Ord for Person {
    fn cmp(&self, other: &Self) -> Ordering {
        self.age.cmp(&other.age)
    }
}

impl PartialOrd for Person {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn main() {
    let mut people = vec![
        Person { name: "Alice".to_string(), age: 25 },
        Person { name: "Bob".to_string(), age: 30 },
        Person { name: "Charlie".to_string(), age: 22 }
    ];

    people.sort();

    for person in &people {
        println!("{:?}", person);
    }
}
