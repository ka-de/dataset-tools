// Turn clippy into a real bitch
#![warn(clippy::all, clippy::pedantic)]

use std::env;
use std::fs::File;
use std::fs::write;
use std::path::Path;
use safetensors::tensor::SafeTensors;
use memmap2::Mmap;
use serde_json::{ Value, Map };
use anyhow::{ Context, Result };
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename or directory>", args[0]);
        return Ok(());
    }
    let path = &args[1];
    if Path::new(path).is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if
                entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str()) == Some("safetensors")
            {
                if let Err(err) = process_file(entry.path()) {
                    eprint!("Error processing {:#?}: {}", entry.path(), err);
                }
            }
        }
    } else {
        process_file(Path::new(path))?;
    }
    Ok(())
}

fn process_file(path: &Path) -> Result<()> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    let json = get_json_metadata(&mmap)?;
    let pretty_json = serde_json::to_string_pretty(&json)?;

    println!("{}", pretty_json);
    write(path.with_extension("json"), pretty_json)?;
    Ok(())
}

fn get_json_metadata(buffer: &[u8]) -> Result<Value> {
    let (_header_size, metadata) =
        SafeTensors::read_metadata(buffer).context("Cannot read metadata")?;
    let metadata = metadata.metadata().as_ref().context("No metadata available")?;

    let mut kv = Map::with_capacity(metadata.len());
    for (key, value) in metadata {
        let json_value = serde_json::from_str(value).unwrap_or_else(|_| {
            // Converts few python literals, then bail out by interpreting the value as a string
            match value.as_str() {
                "True" => Value::Bool(true),
                "False" => Value::Bool(false),
                "None" => Value::Null,
                s => Value::String(s.into()),
            }
        });
        kv.insert(key.clone(), json_value);
    }
    Ok(Value::Object(kv))
}
