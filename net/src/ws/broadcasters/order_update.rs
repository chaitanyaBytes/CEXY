use futures_util::StreamExt;
use redis::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ws::client_manager::UserManager;

pub async fn broadcast_order_update_events(
    user_manager: Arc<RwLock<UserManager>>,
    redis_client: Client,
) -> redis::RedisResult<()> {
    let (mut sink, mut stream) = redis_client.get_async_pubsub().await?.split();
    sink.psubscribe("market:order:user:*").await?;

    while let Some(msg) = stream.next().await {
        let channel: String = msg.get_channel().unwrap_or_default();
        let payload: String = msg.get_payload()?;

        if let Some(user_id_str) = channel.rsplit(':').next() {
            if let Ok(user_id) = user_id_str.parse::<u64>() {
                let mut manager = user_manager.write().await;
                manager.send_order_update(user_id, &payload).await;
            } else {
                eprintln!("[order_update] invalid user_id in channel: {}", channel);
            }
        }
    }
    Ok(())
}
