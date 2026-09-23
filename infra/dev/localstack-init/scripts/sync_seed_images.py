#!/usr/bin/env python3
"""
sync_seed_images.py
Synchronizes wine catalog images into LocalStack S3 and guarantees
consistent product_image records in PostgreSQL on container startup.
"""

import os
import sys
import time
import json
import uuid
import hashlib
import urllib.request
import urllib.error

# Configuration
S3_ENDPOINT = os.environ.get("S3_ENDPOINT", "http://localhost:4566")
BUCKET = os.environ.get("S3_BUCKET_NAME", "novgorod-media-dev")
STORAGE_PROVIDER = "s3"
TENANT_ID = int(os.environ.get("TENANT_ID", 1))

POSTGRES_HOST = os.environ.get("POSTGRES_HOST", "postgres")
POSTGRES_PORT = int(os.environ.get("POSTGRES_PORT", 5432))
POSTGRES_DB = os.environ.get("POSTGRES_DB", "novgorod-dev")
POSTGRES_USER = os.environ.get("POSTGRES_USER", "novgorod-dev")
POSTGRES_PASSWORD = os.environ.get("POSTGRES_PASSWORD", "9e374511")

# Locate manifest and images directory
POSSIBLE_PATHS = [
    "/data/images/images_manifest.json",
    os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "data", "wines", "images", "images_manifest.json")),
    "/home/erick/workspace/novgorod/data/wines/images/images_manifest.json"
]

MANIFEST_PATH = None
for path in POSSIBLE_PATHS:
    if os.path.exists(path):
        MANIFEST_PATH = path
        break

if not MANIFEST_PATH:
    print(f"[sync_seed_images] Error: images_manifest.json not found in candidate paths.")
    sys.exit(0)

IMAGES_DIR = os.path.dirname(MANIFEST_PATH)

def compute_storage_identity_hash(provider: str, bucket: str, object_key: str) -> bytes:
    h = hashlib.sha256()
    h.update(provider.encode("utf-8"))
    h.update(b":")
    h.update(bucket.encode("utf-8"))
    h.update(b":")
    h.update(object_key.encode("utf-8"))
    return h.digest()

def upload_file_to_s3(local_path: str, object_key: str, mime_type: str = "image/jpeg"):
    url = f"{S3_ENDPOINT}/{BUCKET}/{object_key}"
    with open(local_path, "rb") as f:
        data = f.read()

    req = urllib.request.Request(url, data=data, method="PUT")
    req.add_header("Content-Type", mime_type)
    with urllib.request.urlopen(req, timeout=15) as resp:
        if resp.status not in (200, 201):
            raise Exception(f"S3 PUT returned status {resp.status}")

def get_db_connection():
    try:
        import pg8000.native
    except ImportError:
        print("[sync_seed_images] Warning: pg8000 not installed, skipping DB sync and uploading to S3 with deterministic keys only.")
        return None

    # Wait for PostgreSQL to be accessible (up to 15 seconds)
    conn = None
    for attempt in range(15):
        try:
            conn = pg8000.native.Connection(
                user=POSTGRES_USER,
                host=POSTGRES_HOST,
                port=POSTGRES_PORT,
                database=POSTGRES_DB,
                password=POSTGRES_PASSWORD,
                timeout=5
            )
            break
        except Exception as e:
            time.sleep(1)

    if not conn:
        print(f"[sync_seed_images] Warning: Could not connect to PostgreSQL at {POSTGRES_HOST}:{POSTGRES_PORT}")
    return conn

def sync():
    print(f"[sync_seed_images] Starting synchronization using manifest: {MANIFEST_PATH}")
    with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
        manifest = json.load(f)

    conn = get_db_connection()
    db_images = {} # (product_id, sort_order) -> (object_key, storage_status)
    db_table_exists = False

    if conn:
        try:
            # Check if table product_image exists
            res = conn.run("SELECT to_regclass('public.product_image');")
            if res and res[0][0]:
                db_table_exists = True
                rows = conn.run(
                    "SELECT product_id, sort_order, object_key, storage_status "
                    "FROM product_image WHERE tenant_id = :tenant_id;",
                    tenant_id=TENANT_ID
                )
                for r in rows:
                    p_id, s_order, o_key, s_status = r[0], r[1], r[2], r[3]
                    db_images[(p_id, s_order)] = (o_key, s_status)
                print(f"[sync_seed_images] Found {len(db_images)} product_image records in database.")
            else:
                print("[sync_seed_images] Table 'product_image' does not exist yet (migrations pending).")
        except Exception as e:
            print(f"[sync_seed_images] Error querying database: {e}")

    total_uploaded = 0
    total_db_updated = 0
    total_db_inserted = 0

    for prod in manifest:
        product_id = prod["product_id"]
        product_key = prod["product_key"]
        product_name = prod["product_name"]
        brand = prod.get("brand", "")

        for idx, img in enumerate(prod.get("images", [])):
            file_name = img["file_name"]
            sort_order = img.get("sort_order", idx)
            is_primary = (sort_order == 0)

            # Local file resolution
            local_path = os.path.join(IMAGES_DIR, product_key, file_name)
            if not os.path.exists(local_path):
                # Fallback to absolute path from manifest if present
                if "path" in img and os.path.exists(img["path"]):
                    local_path = img["path"]
                else:
                    continue

            size_bytes = os.path.getsize(local_path)
            width_px = img.get("width", 800)
            height_px = img.get("height", 800)

            # Deterministic UUID for the image
            det_uuid = uuid.uuid5(uuid.NAMESPACE_DNS, f"novgorod-t{TENANT_ID}-p{product_id}-img{sort_order}")
            uuid_bytes = det_uuid.bytes
            uuid_hex = det_uuid.hex
            default_object_key = f"tenants/{TENANT_ID}/products/{product_id}/images/{uuid_hex}-{file_name}"

            # Determine target object key: if database already has a key for this image, preserve it!
            if (product_id, sort_order) in db_images:
                object_key, current_status = db_images[(product_id, sort_order)]
            else:
                object_key = default_object_key
                current_status = None

            storage_hash = compute_storage_identity_hash(STORAGE_PROVIDER, BUCKET, object_key)

            # 1. Upload to LocalStack S3
            try:
                upload_file_to_s3(local_path, object_key, "image/jpeg")
                total_uploaded += 1
            except Exception as e:
                print(f"[sync_seed_images] Error uploading {local_path} -> s3://{BUCKET}/{object_key}: {e}")
                continue

            # 2. Synchronize database if connection and table exist
            if conn and db_table_exists:
                try:
                    if (product_id, sort_order) in db_images:
                        if current_status != "available":
                            conn.run(
                                "UPDATE product_image SET storage_status = 'available', updated_at = NOW() "
                                "WHERE tenant_id = :tenant_id AND product_id = :product_id AND sort_order = :sort_order;",
                                tenant_id=TENANT_ID, product_id=product_id, sort_order=sort_order
                            )
                            total_db_updated += 1
                    else:
                        alt_text = f"{product_name} - {brand} (Foto {sort_order + 1})"[:255]
                        conn.run(
                            """
                            INSERT INTO product_image (
                                uuid, tenant_id, product_id, sku_id, alt_text, sort_order,
                                is_primary, storage_provider, bucket, object_key,
                                storage_identity_hash, object_version, etag, checksum_sha256,
                                original_filename, mime_type, size_bytes, width_px, height_px,
                                storage_status, created_at, created_by, updated_at, updated_by
                            ) VALUES (
                                :uuid, :tenant_id, :product_id, NULL, :alt_text, :sort_order,
                                :is_primary, :storage_provider, :bucket, :object_key,
                                :storage_identity_hash, NULL, NULL, NULL,
                                :original_filename, 'image/jpeg', :size_bytes, :width_px, :height_px,
                                'available', NOW(), 'init_hook_sync', NOW(), 'init_hook_sync'
                            ) ON CONFLICT (storage_identity_hash) DO UPDATE
                            SET storage_status = 'available', updated_at = NOW();
                            """,
                            uuid=uuid_bytes,
                            tenant_id=TENANT_ID,
                            product_id=product_id,
                            alt_text=alt_text,
                            sort_order=sort_order,
                            is_primary=is_primary,
                            storage_provider=STORAGE_PROVIDER,
                            bucket=BUCKET,
                            object_key=object_key,
                            storage_identity_hash=storage_hash,
                            original_filename=file_name,
                            size_bytes=size_bytes,
                            width_px=width_px,
                            height_px=height_px
                        )
                        total_db_inserted += 1
                except Exception as e:
                    print(f"[sync_seed_images] Error syncing DB for product {product_id}, sort_order {sort_order}: {e}")

    if conn:
        conn.close()

    print(f"[sync_seed_images] Success! Uploaded {total_uploaded} images to S3 (bucket: '{BUCKET}').")
    if db_table_exists:
        print(f"[sync_seed_images] Database state: {total_db_inserted} inserted, {total_db_updated} updated, all images marked 'available'.")

if __name__ == "__main__":
    sync()
