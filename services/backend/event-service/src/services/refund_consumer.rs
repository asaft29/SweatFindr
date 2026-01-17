use crate::repositories::refund_repo::RefundRepo;
use anyhow::{Context, Result};
use common::rabbitmq::RabbitMQ;
use common::rabbitmq::messages::{
    QUEUE_REFUND_REQUESTED, ROUTING_KEY_REFUND_REQUESTED, RefundRequested,
};
use common::websocket::messages::{
    NewRefundRequest, ROUTING_KEY_WS_BROADCAST, RefundStatusChanged, WebSocketMessage,
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

    pub async fn start(&self) -> Result<()> {
        self.rabbitmq
            .declare_queue(QUEUE_REFUND_REQUESTED, ROUTING_KEY_REFUND_REQUESTED)
            .await
            .context("Failed to declare queue")?;

        let channel = self
            .rabbitmq
            .get_channel()
            .await
            .context("Channel not available")?;

        let consumer = channel
            .basic_consume(
                QUEUE_REFUND_REQUESTED,
                "event-service-refund-request-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("Failed to create consumer")?;

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
    ) -> Result<()> {
        let created_refund = refund_repo
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
            .map_err(|e| anyhow::anyhow!("Failed to create refund request: {:?}", e))?;

        let ws_message_owner = WebSocketMessage::NewRefundRequest(NewRefundRequest {
            request_id: created_refund.id,
            ticket_cod: created_refund.ticket_cod.clone(),
            requester_email: created_refund.requester_email.clone(),
            event_id: created_refund.event_id,
            packet_id: created_refund.packet_id,
            reason: created_refund.reason.clone().unwrap_or_default(),
            created_at: created_refund
                .created_at
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            event_owner_id: created_refund.event_owner_id,
        });

        if let Ok(json) = serde_json::to_vec(&ws_message_owner) {
            if let Err(e) = self.rabbitmq.publish(ROUTING_KEY_WS_BROADCAST, &json).await {
                error!(
                    "Failed to publish WebSocket notification for new refund request: {:?}",
                    e
                );
            } else {
                info!(
                    "Published WebSocket notification to owner for refund request {}",
                    created_refund.id
                );
            }
        }

        let ws_message_client = WebSocketMessage::RefundStatusChanged(RefundStatusChanged {
            request_id: created_refund.id,
            ticket_cod: created_refund.ticket_cod.clone(),
            status: "PENDING".to_string(),
            event_name: None,
            message: Some("Your refund request has been submitted".to_string()),
            user_id: created_refund.requester_id,
        });

        if let Ok(json) = serde_json::to_vec(&ws_message_client) {
            if let Err(e) = self.rabbitmq.publish(ROUTING_KEY_WS_BROADCAST, &json).await {
                error!(
                    "Failed to publish WebSocket notification for client: {:?}",
                    e
                );
            } else {
                info!(
                    "Published WebSocket notification to client for refund request {}",
                    created_refund.id
                );
            }
        }

        Ok(())
    }
}
