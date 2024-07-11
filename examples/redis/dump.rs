use redis::{ AsyncCommands, aio::MultiplexedConnection };

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Open a connection to the Redis server.
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    // Get all keys.
    let keys: Vec<String> = con.keys("*").await?;

    // Iterate over the keys and get their values.
    for key in keys {
        let value: String = con.get(&key).await?;
        println!("{key}: {value}");
    }

    Ok(())
}
