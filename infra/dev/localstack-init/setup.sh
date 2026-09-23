#!/bin/bash
set -euo pipefail

echo "=========================================================="
echo "Initializing LocalStack Resources & Wine Media for Novgorod"
echo "=========================================================="

BUCKET_NAME="${S3_BUCKET_NAME:-novgorod-media-dev}"
QUEUE_NAME="${SQS_QUEUE_NAME:-product-image-uploads}"
REGION="${AWS_DEFAULT_REGION:-us-east-1}"

# 1. Create S3 Bucket
echo "[1/5] Creating S3 bucket: ${BUCKET_NAME}"
awslocal s3 mb "s3://${BUCKET_NAME}" --region "${REGION}" || true

# 2. Configure S3 CORS for web browser uploads and asset loading
echo "[2/5] Configuring CORS on bucket: ${BUCKET_NAME}"
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
awslocal s3api put-bucket-cors --bucket "${BUCKET_NAME}" --cors-configuration file:///tmp/cors-config.json

# 3. Create SQS Queue
echo "[3/5] Creating SQS queue: ${QUEUE_NAME}"
awslocal sqs create-queue --queue-name "${QUEUE_NAME}" --region "${REGION}" || true

QUEUE_URL=$(awslocal sqs get-queue-url --queue-name "${QUEUE_NAME}" --query 'QueueUrl' --output text)
QUEUE_ARN=$(awslocal sqs get-queue-attributes --queue-url "${QUEUE_URL}" --attribute-names QueueArn --query 'Attributes.QueueArn' --output text)
echo "Queue URL: ${QUEUE_URL}, Queue ARN: ${QUEUE_ARN}"

# 4. Set SQS policy & S3 bucket event notifications
echo "[4/5] Configuring SQS policy and S3 ObjectCreated notification..."
cat << EOF > /tmp/sqs-policy.json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": "*",
      "Action": "sqs:SendMessage",
      "Resource": "${QUEUE_ARN}"
    }
  ]
}
EOF
python3 -c "import json; policy = open('/tmp/sqs-policy.json').read(); open('/tmp/sqs-attributes.json', 'w').write(json.dumps({'Policy': policy}))"
awslocal sqs set-queue-attributes --queue-url "${QUEUE_URL}" --attributes file:///tmp/sqs-attributes.json || true

cat << EOF > /tmp/s3-notification.json
{
  "QueueConfigurations": [
    {
      "QueueArn": "${QUEUE_ARN}",
      "Events": ["s3:ObjectCreated:*"]
    }
  ]
}
EOF
awslocal s3api put-bucket-notification-configuration --bucket "${BUCKET_NAME}" --notification-configuration file:///tmp/s3-notification.json

# 5. Offline install pg8000 and run image sync
echo "[5/5] Synchronizing wine media images to S3 & verifying database..."
if [ -d "/etc/localstack/init/ready.d/vendor" ]; then
    python3 -m pip install --quiet --no-index --find-links=/etc/localstack/init/ready.d/vendor pg8000 || true
fi

if [ -f "/etc/localstack/init/ready.d/scripts/sync_seed_images.py" ]; then
    python3 /etc/localstack/init/ready.d/scripts/sync_seed_images.py
fi

echo "=========================================================="
echo "Novgorod LocalStack & Media initialization complete!"
echo "=========================================================="
