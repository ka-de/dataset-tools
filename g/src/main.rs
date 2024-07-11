// g/src/main.rs

use std::{ process::exit, fs, path::Path, env };
use getopts::Options;
use ::redis::Client;

use g::{ build_octocrab, github, redis };

/// The entry point of the application.
///
/// This function parses command-line arguments and executes the corresponding command.
/// It handles errors by printing them to standard error and exiting the program with a non-zero status code.
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        exit(1);
    }
}

/// Runs the main logic of the application based on parsed command-line arguments.
///
/// # Errors
///
/// Returns an error if there is an issue with parsing command-line arguments,
/// interacting with the GitHub API, or communicating with the Redis server.
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("", "public", "make the gist public");
    opts.optflag("", "private", "make the gist private");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return Ok(());
    }

    let token = env::var("GITHUB_TOKEN").map_err(|_| "GITHUB_TOKEN env variable is required")?;
    let octocrab = build_octocrab(&token)?;

    match matches.free.first().map(String::as_str) {
        Some("store-repos") => {
            let repos = github::list_repos(&octocrab).await?;
            let client = Client::open("redis://127.0.0.1/")?;
            let mut con = client.get_multiplexed_async_connection().await?;
            redis::store_repos(&mut con, &repos).await?;
            println!("Repositories stored in Redis");
        }
        Some("repo-stats") => {
            if matches.free.len() < 3 {
                eprintln!("Not enough arguments for repo-stats command");
                print_usage(&program, &opts);
                exit(1);
            }
            let owner = &matches.free[1];
            let repo = &matches.free[2];
            let (full_name, stars, health_percentage) = github::get_repo_stats(
                &octocrab,
                owner,
                repo
            ).await?;
            println!("{full_name} has {stars} stars and {health_percentage}% health percentage");
        }
        Some("gist") => {
            if matches.free.len() < 3 {
                eprintln!("Not enough arguments for gist command");
                print_usage(&program, &opts);
                exit(1);
            }

            let file_path = &matches.free[1];
            let description = &matches.free[2];
            let is_public = !matches.opt_present("private");

            let file_name = Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path);

            let content = fs::read_to_string(file_path)?;

            let url = github::create_gist(
                &octocrab,
                file_name,
                &content,
                description,
                is_public
            ).await?;
            println!("Gist created: {url}");
        }
        Some("list-repos") => {
            let repos = github::list_repos(&octocrab).await?;
            for (name, url) in repos {
                println!("{name}: {url}");
            }
        }
        _ => {
            print_usage(&program, &opts);
            exit(1);
        }
    }

    Ok(())
}

/// Prints the usage information for the application.
///
/// This function displays the available commands and options to the user.
///
/// # Arguments
///
/// * `program` - The name of the program executable.
/// * `opts` - The `Options` instance containing the command-line options.
fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {program} [options] COMMAND");
    print!("{}", opts.usage(&brief));
    println!("\nCommands:");
    println!("  gist <file_path> <description>  Create a gist");
    println!("  list-repos                      List user repositories");
    println!("  store-repos                     Store user repositories in Redis");
    println!("  repo-stats <owner> <repo>       Get statistics for a specific repository");
}
