use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Open a connection to the Redis server.
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con: MultiplexedConnection = client.get_multiplexed_async_connection().await?;

    // Set a key-value pair in Redis.
    let _: () = con.set("wolf", "awoo").await?;

    println!("Key set successfully");

    Ok(())
}
