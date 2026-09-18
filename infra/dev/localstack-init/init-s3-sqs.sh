#!/bin/bash
set -euo pipefail

echo "Initializing LocalStack S3 & SQS resources for Novgorod..."

BUCKET_NAME="${S3_BUCKET_NAME:-novgorod-media-dev}"
QUEUE_NAME="${SQS_QUEUE_NAME:-product-image-uploads}"
REGION="${AWS_DEFAULT_REGION:-us-east-1}"

# 1. Create S3 Bucket
echo "Creating S3 bucket: $BUCKET_NAME"
awslocal s3 mb "s3://$BUCKET_NAME" --region "$REGION" || true

# 2. Configure S3 CORS to allow direct PUT uploads from frontend
echo "Configuring CORS on S3 bucket: $BUCKET_NAME"
cat << 'EOF' > /tmp/cors-config.json
{
  "CORSRules": [
    {
      "AllowedOrigins": ["*"],
      "AllowedMethods": ["GET", "PUT", "POST", "DELETE", "HEAD"],
      "AllowedHeaders": ["*"],
      "ExposeHeaders": ["ETag"],
      "MaxAgeSeconds": 3000
    }
  ]
}
EOF
awslocal s3api put-bucket-cors --bucket "$BUCKET_NAME" --cors-configuration file:///tmp/cors-config.json

# 3. Create SQS Queue
echo "Creating SQS queue: $QUEUE_NAME"
awslocal sqs create-queue --queue-name "$QUEUE_NAME" --region "$REGION" || true

# 4. Get SQS Queue ARN
QUEUE_URL=$(awslocal sqs get-queue-url --queue-name "$QUEUE_NAME" --query 'QueueUrl' --output text)
QUEUE_ARN=$(awslocal sqs get-queue-attributes --queue-url "$QUEUE_URL" --attribute-names QueueArn --query 'Attributes.QueueArn' --output text)
echo "Queue URL: $QUEUE_URL, Queue ARN: $QUEUE_ARN"

# 5. Set SQS policy allowing S3 to send messages
cat << EOF > /tmp/sqs-policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": "sqs:SendMessage",
      "Resource": "$QUEUE_ARN"
    }
  ]
}
EOF
python3 -c "import json; policy = open('/tmp/sqs-policy.json').read(); open('/tmp/sqs-attributes.json', 'w').write(json.dumps({'Policy': policy}))"
awslocal sqs set-queue-attributes --queue-url "$QUEUE_URL" --attributes file:///tmp/sqs-attributes.json || true


# 6. Configure S3 bucket notification to send s3:ObjectCreated:* to SQS queue
echo "Configuring S3 notification to SQS..."
cat << EOF > /tmp/s3-notification.json
{
  "QueueConfigurations": [
    {
      "QueueArn": "$QUEUE_ARN",
      "Events": ["s3:ObjectCreated:*"]
    }
  ]
}
EOF
awslocal s3api put-bucket-notification-configuration --bucket "$BUCKET_NAME" --notification-configuration file:///tmp/s3-notification.json

echo "LocalStack S3 & SQS resources successfully configured!"
