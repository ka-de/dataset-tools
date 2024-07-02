use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 { &args[1] } else { "." };

    println!("Searching for .URL files in: {}", target_dir);

    for entry in WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.eq_ignore_ascii_case("url") {
                    match fs::remove_file(path) {
                        Ok(_) => println!("Removed: {}", path.display()),
                        Err(e) => eprintln!("Error removing {}: {}", path.display(), e),
                    }
                }
            }
        }
    }

    println!("Search complete.");
}
