use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client as S3Client;
use std::env;
use std::time::Duration;

#[derive(Clone)]
pub struct StorageGateway {
    bucket_name: String,
    cdn_base_url: String,
    client: S3Client,
}

impl StorageGateway {
    pub fn new(bucket_name: String, cdn_base_url: String, client: S3Client) -> Self {
        Self {
            bucket_name,
            cdn_base_url: cdn_base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    pub async fn from_env() -> Self {
        let region = env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let bucket_name = env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "novgorod-media-dev".to_string());
        let cdn_base_url = env::var("CDN_BASE_URL").unwrap_or_else(|_| format!("http://localhost:4566/{}", bucket_name));
        
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region));

        if let Ok(endpoint_url) = env::var("AWS_ENDPOINT_URL") {
            if !endpoint_url.is_empty() {
                config_loader = config_loader.endpoint_url(endpoint_url);
            }
        }

        let sdk_config = config_loader.load().await;
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);
        
        if env::var("AWS_ENDPOINT_URL").is_ok() {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let client = S3Client::from_conf(s3_config_builder.build());

        Self::new(bucket_name, cdn_base_url, client)
    }

    pub fn bucket_name(&self) -> &str {
        &self.bucket_name
    }

    pub fn get_cdn_url(&self, object_key: &str) -> String {
        let clean_key = object_key.trim_start_matches('/');
        format!("{}/{}", self.cdn_base_url, clean_key)
    }

    pub async fn generate_presigned_upload_url(
        &self,
        object_key: &str,
        content_type: &str,
        expires_in_secs: u64,
    ) -> Result<String, String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_in_secs))
            .build()
            .map_err(|e| format!("Failed to build presigning config: {}", e))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .key(object_key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Failed to presign S3 PUT URL: {}", e))?;

        Ok(presigned.uri().to_string())
    }

    pub async fn delete_object(&self, object_key: &str) -> Result<(), String> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(object_key)
            .send()
            .await
            .map_err(|e| format!("Failed to delete S3 object {}: {}", object_key, e))?;

        Ok(())
    }
}
