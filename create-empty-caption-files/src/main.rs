use std::fs::File;
use walkdir::WalkDir;

fn create_caption_file(directory: &str) {
    for entry in WalkDir::new(directory) {
        let entry = entry.unwrap();
        let path = entry.path();
        let extension = path.extension().unwrap_or_default();
        if ["jpg", "jpeg", "png"].contains(&extension.to_str().unwrap()) {
            let caption_file = path.with_extension("txt");
            if !caption_file.exists() {
                File::create(caption_file).unwrap();
            }
        }
    }
}

fn main() {
    let directory = "E:\\training_dir_staging";
    create_caption_file(directory);
    println!("Caption files created successfully.");
}
