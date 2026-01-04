use lapin::{
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
    options::{BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

pub mod messages;

const REFUND_EXCHANGE: &str = "refund.exchange";

pub struct RabbitMQ {
    connection: Arc<RwLock<Option<Connection>>>,
    channel: Arc<RwLock<Option<Channel>>>,
}

impl RabbitMQ {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(RwLock::new(None)),
            channel: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn connect(&self, url: &str) -> Result<(), lapin::Error> {
        let connection = Connection::connect(url, ConnectionProperties::default()).await?;
        info!("RabbitMQ connection established");

        let channel = connection.create_channel().await?;
        info!("RabbitMQ channel created");

        channel
            .exchange_declare(
                REFUND_EXCHANGE,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
        info!("Exchange '{}' declared", REFUND_EXCHANGE);

        *self.connection.write().await = Some(connection);
        *self.channel.write().await = Some(channel);

        Ok(())
    }

    pub async fn declare_queue(
        &self,
        queue_name: &str,
        routing_key: &str,
    ) -> Result<(), lapin::Error> {
        let channel_guard = self.channel.read().await;
        let channel = channel_guard.as_ref().ok_or_else(|| {
            error!("Channel not initialized");
            lapin::Error::InvalidChannel(0)
        })?;

        channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;

        channel
            .queue_bind(
                queue_name,
                REFUND_EXCHANGE,
                routing_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        info!(
            "Queue '{}' declared and bound to '{}'",
            queue_name, routing_key
        );
        Ok(())
    }

    pub async fn publish(&self, routing_key: &str, message: &[u8]) -> Result<(), lapin::Error> {
        let channel_guard = self.channel.read().await;
        let channel = channel_guard.as_ref().ok_or_else(|| {
            error!("Channel not initialized");
            lapin::Error::InvalidChannel(0)
        })?;

        channel
            .basic_publish(
                REFUND_EXCHANGE,
                routing_key,
                BasicPublishOptions::default(),
                message,
                BasicProperties::default()
                    .with_content_type("application/json".into())
                    .with_delivery_mode(2),
            )
            .await?
            .await?;

        info!("Message published to '{}'", routing_key);
        Ok(())
    }

    pub async fn get_channel(&self) -> Option<Channel> {
        self.channel.read().await.clone()
    }
}

impl Default for RabbitMQ {
    fn default() -> Self {
        Self::new()
    }
}
