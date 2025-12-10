use futures_util::StreamExt;
use redis::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ws::client_manager::UserManager;

pub async fn broadcast_trade_events(
    user_manager: Arc<RwLock<UserManager>>,
    redis_client: Client,
) -> redis::RedisResult<()> {
    let (mut sink, mut stream) = redis_client.get_async_pubsub().await?.split();
    sink.psubscribe("market:trade:*").await?;

    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel().unwrap_or_default();
        let payload: String = msg.get_payload()?;
        let symbol = channel.rsplit(':').next().unwrap_or_default();

        let mut manager = user_manager.write().await;
        manager.broadcast_trade(symbol, &payload).await;
    }

    Ok(())
}
