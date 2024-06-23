use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let root_dir = r"E:\training_dir\_";
    let mut missing_captions = Vec::new();

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok()) {
        let path = entry.path();
        if is_image_file(path) {
            let caption_path = path.with_extension("txt");
            if !caption_file_exists_and_not_empty(&caption_path) {
                missing_captions.push(path.to_string_lossy().to_string());
            }
        }
    }

    if missing_captions.is_empty() {
        println!("All image files have corresponding non-empty caption files.");
    } else {
        println!(
            "The following image files are missing caption files or have empty caption files:"
        );
        for path in missing_captions {
            println!("{}", path);
        }
    }

    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png"),
        None => false,
    }
}

fn caption_file_exists_and_not_empty(path: &Path) -> bool {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => !content.trim().is_empty(),
            Err(_) => false,
        }
    } else {
        false
    }
}
