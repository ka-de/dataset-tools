use octocrab::Octocrab;
use std::process::exit;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(token) = std::env::var("GITHUB_TOKEN") else {
        eprintln!("Error: GITHUB_TOKEN environment variable is not set");
        return Err("Missing GITHUB_TOKEN".into());
    };

    let octocrab = Octocrab::builder().personal_token(token).build()?;
    let repo = octocrab.repos("ka-de", "dataset-tools").get().await?;
    let repo_metrics = octocrab
        .repos("ka-de", "dataset-tools")
        .get_community_profile_metrics().await?;

    println!(
        "{} has {} stars and {}% health percentage",
        repo.full_name.unwrap(),
        repo.stargazers_count.unwrap_or(0),
        repo_metrics.health_percentage
    );

    Ok(())
}
