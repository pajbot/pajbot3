use std::sync::Arc;

use dashmap::DashMap;
use twitch_api::eventsub::channel::chat::message::ChannelChatMessageV1Payload;
use twitch_api::types::UserId;

pub struct Events {
    pub on_chat_message:
        DashMap<UserId, tokio::sync::broadcast::Sender<Arc<ChannelChatMessageV1Payload>>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            on_chat_message: DashMap::new(),
        }
    }

    pub fn get_receiver_chat_message(
        &self,
        broadcaster_id: UserId,
    ) -> anyhow::Result<tokio::sync::broadcast::Receiver<Arc<ChannelChatMessageV1Payload>>> {
        let tx = self
            .on_chat_message
            .entry(broadcaster_id)
            .or_insert_with(|| {
                let (tx, _) = tokio::sync::broadcast::channel(69);
                tx
            });
        Ok(tx.subscribe())
    }

    pub fn publish_chat_message(&self, payload: ChannelChatMessageV1Payload) -> anyhow::Result<()> {
        let broadcaster_id = payload.broadcaster_user_id.clone();
        let tx = self
            .on_chat_message
            .entry(broadcaster_id)
            .or_insert_with(|| {
                let (tx, _) = tokio::sync::broadcast::channel(69);
                tx
            });
        tx.send(Arc::new(payload))?;
        Ok(())
    }
}
