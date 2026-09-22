use aws_sdk_sqs::Client as SqsClient;
use business::gateway::product_image_gateway::ProductImageGateway;
use business::gateway::storage_gateway::StorageGateway;
use business::sea_orm::DatabaseConnection;
use business::use_cases::product_image_use_case::ProductImageUseCase;
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct S3Notification {
    #[serde(rename = "Records", default)]
    records: Vec<S3NotificationRecord>,
}

#[derive(Debug, Deserialize)]
struct S3NotificationRecord {
    #[serde(rename = "eventName", default)]
    event_name: String,
    s3: Option<S3Entity>,
}

#[derive(Debug, Deserialize)]
struct S3Entity {
    bucket: S3BucketEntity,
    object: S3ObjectEntity,
}

#[derive(Debug, Deserialize)]
struct S3BucketEntity {
    name: String,
}

#[derive(Debug, Deserialize)]
struct S3ObjectEntity {
    key: String,
    size: Option<i64>,
    #[serde(rename = "eTag")]
    etag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SnsEnvelope {
    #[serde(rename = "Message")]
    message: String,
}

pub fn spawn_sqs_consumer(db: DatabaseConnection, storage: Arc<StorageGateway>) {
    let queue_url = match env::var("SQS_QUEUE_URL") {
        Ok(url) if !url.trim().is_empty() => url,
        _ => {
            log::info!("SQS_QUEUE_URL not configured; skipping SQS consumer worker");
            return;
        }
    };

    tokio::spawn(async move {
        log::info!("Starting SQS consumer worker for queue: {}", queue_url);

        let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region));

        if let Ok(endpoint_url) = env::var("AWS_ENDPOINT_URL")
            && !endpoint_url.is_empty() {
                config_loader = config_loader.endpoint_url(endpoint_url);
            }

        let sdk_config = config_loader.load().await;
        let sqs_client = SqsClient::new(&sdk_config);

        loop {
            let receive_res = sqs_client
                .receive_message()
                .queue_url(&queue_url)
                .max_number_of_messages(10)
                .wait_time_seconds(10)
                .send()
                .await;

            match receive_res {
                Ok(output) => {
                    let messages = output.messages();
                    for msg in messages {
                        if let Some(body) = msg.body() {
                            let parsed_notification: Option<S3Notification> =
                                serde_json::from_str(body).ok().or_else(|| {
                                    if let Ok(sns) = serde_json::from_str::<SnsEnvelope>(body) {
                                        serde_json::from_str(&sns.message).ok()
                                    } else {
                                        None
                                    }
                                });

                            if let Some(notification) = parsed_notification {
                                let gateway = ProductImageGateway::new(db.clone());
                                let use_case = ProductImageUseCase::new(gateway, (*storage).clone());

                                for record in notification.records {
                                    if let Some(s3) = record.s3 {
                                        let bucket = s3.bucket.name;
                                        let object_key = s3.object.key;
                                        let etag = s3.object.etag;
                                        let size = s3.object.size;

                                        log::info!(
                                            "Processing S3 event '{}' for bucket: {}, key: {}",
                                            record.event_name,
                                            bucket,
                                            object_key
                                        );

                                        if !use_case
                                            .handle_s3_upload_notification(&bucket, &object_key, etag, size)
                                            .await
                                        {
                                            log::warn!(
                                                "Failed handling S3 notification for key {}",
                                                object_key
                                            );
                                        }
                                    }
                                }
                            } else {
                                log::debug!("Ignored non-S3 notification message from SQS: {}", body);
                            }

                            if let Some(receipt_handle) = msg.receipt_handle() {
                                let _ = sqs_client
                                    .delete_message()
                                    .queue_url(&queue_url)
                                    .receipt_handle(receipt_handle)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
                Err(err) => {
                    log::warn!("Error receiving SQS messages: {}. Backing off...", err);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
