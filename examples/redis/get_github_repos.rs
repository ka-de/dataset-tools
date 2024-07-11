use redis::AsyncCommands;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;
    let repos: std::collections::HashMap<String, String> = con.hgetall("github_repos").await?;
    for (key, value) in &repos {
        println!("{key}: {value}");
    }
    Ok(())
}
