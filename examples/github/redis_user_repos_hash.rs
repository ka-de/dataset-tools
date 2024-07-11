use octocrab::Octocrab;
use octocrab::models::Repository;
use url::Url;
use redis::AsyncCommands;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = Octocrab::builder().personal_token(token).build()?;

    // Connect to Redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    let mut page = octocrab
        .current()
        .list_repos_for_authenticated_user()
        .per_page(100)
        .send().await?;

    loop {
        for repo in &page.items {
            let name = &repo.name;
            let url = repo.html_url
                .as_ref()
                .unwrap_or(&Url::parse("https://github.com").unwrap())
                .to_string();

            // Store in Redis hash
            let _: () = con.hset("github_repos", name, &url).await?;

            println!("{name}: {url}");
        }

        if let Some(next_page) = octocrab.get_page::<Repository>(&page.next).await? {
            page = next_page;
        } else {
            break;
        }
    }

    Ok(())
}
