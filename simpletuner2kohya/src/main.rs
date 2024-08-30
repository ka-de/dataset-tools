use std::path::{ Path, PathBuf };
use anyhow::{ Context, Result };
use tokio::fs;
use serde_json::Value;
use regex::Regex;
use dataset_tools::{ walk_directory, process_json_file, write_to_file };

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <json_dir> [--by <artist>]", args[0]);
        std::process::exit(1);
    }

    let json_dir = &args[1];
    let artist = args.get(3).cloned();

    json_to_txt(json_dir, artist).await?;

    Ok(())
}

async fn json_to_txt(json_dir: &str, artist: Option<String>) -> Result<()> {
    let json_path = Path::new(json_dir);
    if !json_path.is_dir() {
        eprintln!("The directory {} does not exist.", json_dir);
        return Ok(());
    }

    walk_directory(json_path, "json", |path| async move {
        process_json_file(&path, |json| async move {
            let filename = json["filename"]
                .as_str()
                .context("Failed to get filename")?
                .replace(".png", ".txt");
            let mut caption = json["caption"]
                .as_str()
                .context("Failed to get caption")?
                .replace('\n', " ")
                .replace("**", "")
                .replace('\\', "");

            if let Some(ref artist_name) = artist {
                caption = format!("by {}, {}", artist_name, caption);
            }

            caption = add_comma_after_period(&caption)?;

            let txt_file_path = path.with_file_name(filename);
            write_to_file(&txt_file_path, &caption).await?;

            println!("Converted {} to {}", path.display(), txt_file_path.display());
            Ok(())
        }).await
    }).await?;

    Ok(())
}

fn add_comma_after_period(text: &str) -> Result<String> {
    let mut result = text.to_string();

    // Remove all commas except for the artist tag
    if let Some(artist_tag) = result.find("by ") {
        let (before, after) = result.split_at(artist_tag);
        let (artist, rest) = after.split_once(',').unwrap_or((after, ""));
        result = format!("{}{}{}", before.replace(',', ""), artist, rest.replace(',', ""));
    } else {
        result = result.replace(',', "");
    }

    // Add comma after each period except within quotes and not after numbers
    let period_regex = Regex::new(r#"(?<!["\'])(?<!\d)\.(?!["\'])"#)?;
    result = period_regex.replace_all(&result, ".,").to_string();

    // Remove dot after every number
    let number_regex = Regex::new(r"(\d)\.")?;
    result = number_regex.replace_all(&result, "$1").to_string();

    // Strip out excessive space characters (more than one)
    let space_regex = Regex::new(r"\s+")?;
    result = space_regex.replace_all(&result, " ").to_string();

    Ok(result)
}
