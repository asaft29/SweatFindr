use common::rabbitmq::RabbitMQ;
use common::websocket::messages::*;
use futures::StreamExt;
use lapin::options::*;
use std::sync::Arc;
use tracing::{error, info};

use super::connection_manager::ConnectionManager;

pub struct WebSocketBroker {
    rabbitmq: Arc<RabbitMQ>,
    ws_manager: Arc<ConnectionManager>,
}

impl WebSocketBroker {
    pub fn new(rabbitmq: Arc<RabbitMQ>, ws_manager: Arc<ConnectionManager>) -> Self {
        Self {
            rabbitmq,
            ws_manager,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let channel = self
            .rabbitmq
            .get_channel()
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to get RabbitMQ channel"))?;

        channel
            .queue_declare(
                QUEUE_WS_BROADCAST,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                Default::default(),
            )
            .await?;

        channel
            .queue_bind(
                QUEUE_WS_BROADCAST,
                "refund.exchange",
                ROUTING_KEY_WS_BROADCAST,
                QueueBindOptions::default(),
                Default::default(),
            )
            .await?;

        let mut consumer = channel
            .basic_consume(
                QUEUE_WS_BROADCAST,
                "notification_ws_broker",
                BasicConsumeOptions::default(),
                Default::default(),
            )
            .await?;

        info!(
            "WebSocket broker started, listening to {}",
            QUEUE_WS_BROADCAST
        );

        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    if let Ok(message) = serde_json::from_slice::<WebSocketMessage>(&delivery.data)
                    {
                        self.handle_message(message).await;
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                    } else {
                        error!("Failed to deserialize WebSocket message");
                    }
                }
                Err(e) => error!("RabbitMQ consumer error: {:?}", e),
            }
        }

        Ok(())
    }

    async fn handle_message(&self, message: WebSocketMessage) {
        let json = serde_json::to_string(&message).unwrap();

        match message {
            WebSocketMessage::RefundStatusChanged(ref data) => {
                self.ws_manager.broadcast_to_user(data.user_id, &json).await;
            }
            WebSocketMessage::NewRefundRequest(ref data) => {
                self.ws_manager
                    .broadcast_to_user(data.event_owner_id, &json)
                    .await;
            }
        }
    }
}
