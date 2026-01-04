use crate::repositories::refund_repo::RefundRepo;
use common::rabbitmq::RabbitMQ;
use common::rabbitmq::messages::{
    QUEUE_REFUND_REQUESTED, ROUTING_KEY_REFUND_REQUESTED, RefundRequested,
};
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct RefundRequestConsumer {
    rabbitmq: Arc<RabbitMQ>,
    refund_repo: Arc<RefundRepo>,
}

impl RefundRequestConsumer {
    pub fn new(rabbitmq: Arc<RabbitMQ>, refund_repo: Arc<RefundRepo>) -> Self {
        Self {
            rabbitmq,
            refund_repo,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.rabbitmq
            .declare_queue(QUEUE_REFUND_REQUESTED, ROUTING_KEY_REFUND_REQUESTED)
            .await
            .map_err(|e| format!("Failed to declare queue: {:?}", e))?;

        let channel = self
            .rabbitmq
            .get_channel()
            .await
            .ok_or("Channel not available")?;

        let consumer = channel
            .basic_consume(
                QUEUE_REFUND_REQUESTED,
                "event-service-refund-request-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| format!("Failed to create consumer: {:?}", e))?;

        info!("Started consuming refund request messages");

        let refund_repo = Arc::clone(&self.refund_repo);

        let mut consumer = consumer;
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(delivery) => {
                    let should_ack = match serde_json::from_slice::<RefundRequested>(&delivery.data)
                    {
                        Ok(message) => {
                            info!(
                                "Received refund request for ticket={}, requester_email={}",
                                message.ticket_cod, message.requester_email
                            );

                            match self.process_refund_request(&refund_repo, &message).await {
                                Ok(_) => {
                                    info!(
                                        "Successfully processed refund request for ticket {}",
                                        message.ticket_cod
                                    );
                                    true
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to process refund request: {:?}. Message will be requeued.",
                                        e
                                    );
                                    false
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize refund request message: {:?}. Discarding message.",
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

    async fn process_refund_request(
        &self,
        refund_repo: &RefundRepo,
        message: &RefundRequested,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        refund_repo
            .create_refund_request(
                &message.ticket_cod,
                message.requester_id,
                &message.requester_email,
                message.event_id,
                message.packet_id,
                message.event_owner_id,
                &message.reason,
            )
            .await
            .map_err(|e| format!("Failed to create refund request: {:?}", e))?;

        Ok(())
    }
}
