use crate::services::email_service::EmailService;
use anyhow::{Context, Result};
use common::rabbitmq::messages::{
    RefundResolved, RefundStatus, QUEUE_REFUND_RESOLVED_EMAIL, ROUTING_KEY_REFUND_RESOLVED,
};
use common::rabbitmq::RabbitMQ;
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::types::FieldTable;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct RefundConsumer {
    rabbitmq: Arc<RabbitMQ>,
    email_service: Arc<EmailService>,
}

impl RefundConsumer {
    pub fn new(rabbitmq: Arc<RabbitMQ>, email_service: Arc<EmailService>) -> Self {
        Self {
            rabbitmq,
            email_service,
        }
    }

    pub async fn start(&self) -> Result<()> {
        self.rabbitmq
            .declare_queue(QUEUE_REFUND_RESOLVED_EMAIL, ROUTING_KEY_REFUND_RESOLVED)
            .await
            .context("Failed to declare queue")?;

        let channel = self
            .rabbitmq
            .get_channel()
            .await
            .context("Channel not available")?;

        let consumer = channel
            .basic_consume(
                QUEUE_REFUND_RESOLVED_EMAIL,
                "email-service-refund-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .context("Failed to create consumer")?;

        info!("Started consuming refund resolution messages");

        let email_service = Arc::clone(&self.email_service);

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

                            let email_service = Arc::clone(&email_service);
                            tokio::spawn(async move {
                                if let Err(e) =
                                    Self::send_notification_task(email_service, message).await
                                {
                                    error!("Failed to send refund notification email: {:?}", e);
                                }
                            });

                            true
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize refund message: {:?}. Discarding message.",
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

    async fn send_notification_task(
        email_service: Arc<EmailService>,
        message: RefundResolved,
    ) -> Result<()> {
        let event_name = message.event_name.as_deref().unwrap_or("Unknown Event");

        match message.status {
            RefundStatus::Approved => {
                email_service
                    .send_refund_approved_email(
                        &message.requester_email,
                        &message.ticket_cod,
                        event_name,
                    )
                    .await?;

                info!(
                    "Sent refund approved notification to {}",
                    message.requester_email
                );
            }
            RefundStatus::Rejected => {
                let rejection_reason = message.message.as_deref().unwrap_or("No reason provided");
                email_service
                    .send_refund_rejected_email(
                        &message.requester_email,
                        &message.ticket_cod,
                        event_name,
                        rejection_reason,
                    )
                    .await?;

                info!(
                    "Sent refund rejected notification to {}",
                    message.requester_email
                );
            }
        }

        Ok(())
    }
}
