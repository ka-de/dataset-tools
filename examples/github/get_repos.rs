use octocrab::Octocrab;
use octocrab::models::Repository;
use url::Url;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    let mut page = octocrab
        .current()
        .list_repos_for_authenticated_user()
        .per_page(100)
        .send().await?;

    loop {
        for repo in &page.items {
            println!(
                "{}: {}",
                repo.name,
                repo.html_url.as_ref().unwrap_or(&Url::parse("https://github.com").unwrap())
            );
        }

        if let Some(next_page) = octocrab.get_page::<Repository>(&page.next).await? {
            page = next_page;
        } else {
            break;
        }
    }

    Ok(())
}
