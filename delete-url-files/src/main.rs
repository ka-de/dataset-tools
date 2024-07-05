// delete-url-files/src/main.rs

use std::env;
use std::path::Path;
use dataset_tools::delete_files_with_extension;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let target_dir = if args.len() > 1 { &args[1] } else { "." };

    println!("Searching for .URL files in: {target_dir}");

    delete_files_with_extension(Path::new(target_dir), "url").await?;

    println!("Search complete.");
    Ok(())
}
