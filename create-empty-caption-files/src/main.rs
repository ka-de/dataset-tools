use std::fs::File;
use std::io;
use walkdir::{ DirEntry, WalkDir };

fn is_not_git(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy() != ".git"
}

fn create_caption_file(directory: &str) -> io::Result<()> {
    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_entry(|e| is_not_git(e)) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
            }
        };
        let path = entry.path();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if ["jpg", "jpeg", "png"].contains(&extension) {
            let caption_file = path.with_extension("txt");
            if !caption_file.exists() {
                File::create(caption_file)?;
            }
        }
    }
    Ok(())
}

fn main() {
    let directory = "E:\\training_dir_staging";
    match create_caption_file(directory) {
        Ok(_) => println!("Caption files created successfully."),
        Err(err) => eprintln!("An error occurred: {}", err),
    }
}
