use std::env;
use std::fs::{ self, File };
use std::io::{ BufRead, BufReader, Write };
use std::path::Path;

use serde_json::Value;
use walkdir::WalkDir;

fn process_file(input_path: &Path) -> std::io::Result<()> {
    if input_path.extension().and_then(|s| s.to_str()) == Some("json") {
        let content = fs::read_to_string(input_path)?;
        let json: Value = serde_json::from_str(&content)?;

        if let Value::Object(map) = json {
            let mut tags: Vec<String> = map
                .iter()
                .filter_map(|(key, value)| {
                    if let Value::Number(num) = value {
                        if num.as_f64().unwrap_or(0.0) > 0.5 {
                            Some(key.replace("(", "\\(").replace(")", "\\)"))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            tags.sort_by(|a, b| b.len().cmp(&a.len()));
            tags.truncate(4);

            let output_path = input_path.with_extension("txt");
            let mut output_file = File::create(output_path)?;
            write!(output_file, "{}", tags.join(", "))?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_directory_or_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    if input_path.is_dir() {
        for entry in WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                process_file(path)?;
            }
        }
    } else if input_path.is_file() {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let path = Path::new(&line);
            if path.exists() {
                process_file(path)?;
            } else {
                eprintln!("File not found: {}", line);
            }
        }
    } else {
        eprintln!("Invalid input: not a directory or file");
        std::process::exit(1);
    }

    Ok(())
}
