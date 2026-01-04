use crate::repositories::client_repo::ClientRepo;
use common::rabbitmq::RabbitMQ;
use common::rabbitmq::messages::{
    QUEUE_REFUND_RESOLVED_CLIENT, ROUTING_KEY_REFUND_RESOLVED, RefundResolved, RefundStatus,
};
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct RefundConsumer {
    rabbitmq: Arc<RabbitMQ>,
    client_repo: Arc<ClientRepo>,
}

impl RefundConsumer {
    pub fn new(rabbitmq: Arc<RabbitMQ>, client_repo: Arc<ClientRepo>) -> Self {
        Self {
            rabbitmq,
            client_repo,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.rabbitmq
            .declare_queue(QUEUE_REFUND_RESOLVED_CLIENT, ROUTING_KEY_REFUND_RESOLVED)
            .await
            .map_err(|e| format!("Failed to declare queue: {:?}", e))?;

        let channel = self
            .rabbitmq
            .get_channel()
            .await
            .ok_or("Channel not available")?;

        let consumer = channel
            .basic_consume(
                QUEUE_REFUND_RESOLVED_CLIENT,
                "client-service-refund-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| format!("Failed to create consumer: {:?}", e))?;

        info!("Started consuming refund resolution messages");

        let mut consumer = consumer;
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let should_ack = match serde_json::from_slice::<RefundResolved>(&delivery.data)
                    {
                        Ok(message) => {
                            info!(
                                "Received refund resolution: ticket={}, status={:?}",
                                message.ticket_cod, message.status
                            );

                            match self.process_refund_resolution(&message).await {
                                Ok(_) => {
                                    info!(
                                        "Successfully processed refund resolution for ticket {}",
                                        message.ticket_cod
                                    );
                                    true
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to process refund resolution: {:?}. Message will be requeued.",
                                        e
                                    );
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize refund resolution message: {:?}. Discarding.",
                                e
                            );
                            true
                        }
                    };

                    if should_ack {
                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            error!("Failed to ack message: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Consumer error: {:?}", e);
                }
            }
        }

        Ok(())
    }

    async fn process_refund_resolution(
        &self,
        message: &RefundResolved,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self
            .client_repo
            .find_client_by_ticket_code(&message.ticket_cod)
            .await
            .map_err(|e| format!("Failed to find client: {:?}", e))?
            .ok_or_else(|| format!("No client found with ticket {}", message.ticket_cod))?;

        let client_id = client.id.to_hex();

        match message.status {
            RefundStatus::Approved => {
                self.client_repo
                    .remove_ticket_from_client(&client_id, &message.ticket_cod)
                    .await
                    .map_err(|e| format!("Failed to remove ticket: {:?}", e))?;

                info!(
                    "Removed ticket {} from client {} (refund approved)",
                    message.ticket_cod, client_id
                );
            }
            RefundStatus::Rejected => {
                self.client_repo
                    .update_ticket_refund_status(&client_id, &message.ticket_cod, Some("REJECTED"))
                    .await
                    .map_err(|e| format!("Failed to update ticket status: {:?}", e))?;

                info!(
                    "Marked ticket {} as REJECTED for client {}",
                    message.ticket_cod, client_id
                );
            }
        }

        Ok(())
    }
}
