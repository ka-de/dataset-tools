use octocrab::Octocrab;
use std::env;
use std::fs;
use std::path::Path;
use getopts::Options;

#[tokio::main]
async fn main() {
    let token = match env::var("GITHUB_TOKEN") {
        Ok(val) => val,
        Err(_e) => {
            eprintln!("GITHUB_TOKEN env variable is required");
            std::process::exit(1);
        }
    };

    let octocrab = match Octocrab::builder().personal_token(token).build() {
        Ok(val) => val,
        Err(_e) => {
            eprintln!("Failed to build Octocrab");
            std::process::exit(1);
        }
    };

    // Get the command line arguments
    let args: Vec<String> = env::args().collect();

    // Define the options
    let mut opts = Options::new();
    opts.optflag("", "public", "make the gist public");
    opts.optflag("", "private", "make the gist private");

    // Parse the options
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{f}");
            std::process::exit(1);
        }
    };

    // The first argument is the file path, the second is the description
    let file_path: &String = if matches.free.is_empty() {
        eprintln!("No file path provided");
        std::process::exit(1);
    } else {
        &matches.free[0]
    };
    let description = if matches.free.len() > 1 { &matches.free[1] } else { "" };

    // Determine if the gist should be public or private
    let is_public = if matches.opt_present("private") {
        false
    } else {
        // Default to public if no option is provided
        true
    };

    // Strip the file path from the name
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file_path);

    // Read the file content
    let content = match fs::read_to_string(file_path) {
        Ok(val) => val,
        Err(_e) => {
            eprintln!("Could not read file");
            std::process::exit(1);
        }
    };

    println!("Creating a gist with the content of {file_name} on your account");
    let gist = match
        octocrab
            .gists()
            .create()
            .file(file_name, &content)
            // Optional Parameters
            .description(description)
            .public(is_public)
            .send().await
    {
        Ok(val) => val,
        Err(_e) => {
            eprintln!("Failed to create gist");
            std::process::exit(1);
        }
    };
    println!("Done, created: {url}", url = gist.html_url);
}
