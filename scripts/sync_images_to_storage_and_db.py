#!/usr/bin/env python3
"""
sync_images_to_storage_and_db.py
Uploads downloaded wine images to LocalStack S3 (bucket: novgorod-media-dev)
and registers them into the PostgreSQL product_image table with storage_status='available'.
"""

import os
import sys
import uuid
import hashlib
import json
import subprocess
import urllib.request
from PIL import Image

WORKSPACE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DATA_DIR = os.path.join(WORKSPACE_DIR, "data", "wines")
IMAGES_DIR = os.path.join(DATA_DIR, "images")
MANIFEST_PATH = os.path.join(IMAGES_DIR, "images_manifest.json")

S3_ENDPOINT = "http://localhost:4566"
BUCKET = "novgorod-media-dev"
STORAGE_PROVIDER = "s3"
TENANT_ID = 1

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

def sync_all():
    if not os.path.exists(MANIFEST_PATH):
        print(f"Error: Manifest {MANIFEST_PATH} not found.")
        sys.exit(1)
    
    with open(MANIFEST_PATH, "r", encoding="utf-8") as f:
        manifest = json.load(f)
    
    print(f"Loaded manifest with {len(manifest)} products. Preparing S3 upload and DB sync...")
    
    # Check existing images in database to avoid duplicate insertions
    check_cmd = [
        "docker", "exec", "novgorod_postgres", "psql",
        "-U", "novgorod-dev", "-d", "novgorod-dev", "-t", "-A",
        "-c", "SELECT product_id, sort_order FROM product_image WHERE tenant_id = 1;"
    ]
    check_res = subprocess.run(check_cmd, stdout=subprocess.PIPE, text=True, check=True)
    existing_pairs = set()
    for line in check_res.stdout.strip().split("\n"):
        if "|" in line:
            p_id, s_order = line.split("|")
            existing_pairs.add((int(p_id), int(s_order)))
    
    print(f"Found {len(existing_pairs)} existing images already registered in DB.")
    
    sql_statements = ["BEGIN;"]
    total_uploaded = 0
    total_registered = 0
    
    for prod in manifest:
        product_id = prod["product_id"]
        product_name = prod["product_name"]
        brand = prod["brand"]
        
        for idx, img in enumerate(prod.get("images", [])):
            local_path = img["path"]
            if not os.path.exists(local_path):
                print(f"Warning: file {local_path} not found on disk.")
                continue
            
            sort_order = idx
            is_primary = "true" if idx == 0 else "false"
            
            # Read fresh dimensions and size from disk
            size_bytes = os.path.getsize(local_path)
            try:
                with Image.open(local_path) as im:
                    width_px, height_px = im.size
            except Exception:
                width_px, height_px = img.get("width", 800), img.get("height", 800)
            
            img_uuid = uuid.uuid5(uuid.NAMESPACE_DNS, f"novgorod-t{TENANT_ID}-p{product_id}-img{sort_order}")
            uuid_bytes_hex = img_uuid.hex
            clean_filename = f"image_{idx + 1}.jpg"
            object_key = f"tenants/{TENANT_ID}/products/{product_id}/images/{img_uuid}-{clean_filename}"
            
            storage_hash = compute_storage_identity_hash(STORAGE_PROVIDER, BUCKET, object_key)
            hash_hex = storage_hash.hex()
            
            # 1. Upload to LocalStack S3
            try:
                upload_file_to_s3(local_path, object_key, "image/jpeg")
                total_uploaded += 1
            except Exception as e:
                print(f"Error uploading {local_path} to S3: {e}")
                continue
            
            # 2. Add SQL insert if not already present
            if (product_id, sort_order) in existing_pairs:
                # Update existing record
                sql = f"""
                UPDATE product_image
                SET object_key = '{object_key}',
                    storage_identity_hash = decode('{hash_hex}', 'hex'),
                    size_bytes = {size_bytes},
                    width_px = {width_px},
                    height_px = {height_px},
                    storage_status = 'available',
                    updated_at = NOW()
                WHERE tenant_id = {TENANT_ID} AND product_id = {product_id} AND sort_order = {sort_order};
                """
            else:
                alt_text = f"{product_name} - {brand} (Foto {idx + 1})".replace("'", "''")
                sql = f"""
                INSERT INTO product_image (
                    uuid, tenant_id, product_id, sku_id, alt_text, sort_order,
                    is_primary, storage_provider, bucket, object_key,
                    storage_identity_hash, object_version, etag, checksum_sha256,
                    original_filename, mime_type, size_bytes, width_px, height_px,
                    storage_status, created_at, created_by, updated_at, updated_by
                ) VALUES (
                    decode('{uuid_bytes_hex}', 'hex'),
                    {TENANT_ID},
                    {product_id},
                    NULL,
                    '{alt_text}',
                    {sort_order},
                    {is_primary},
                    '{STORAGE_PROVIDER}',
                    '{BUCKET}',
                    '{object_key}',
                    decode('{hash_hex}', 'hex'),
                    NULL,
                    NULL,
                    NULL,
                    '{clean_filename}',
                    'image/jpeg',
                    {size_bytes},
                    {width_px},
                    {height_px},
                    'available',
                    NOW(),
                    'fabricant_catalog_import',
                    NOW(),
                    'fabricant_catalog_import'
                );
                """
            sql_statements.append(sql.strip())
            total_registered += 1
    
    sql_statements.append("COMMIT;")
    
    print(f"Successfully uploaded {total_uploaded} images to LocalStack S3!")
    print(f"Executing database transaction for {total_registered} product images...")
    
    full_sql = "\n".join(sql_statements)
    psql_cmd = [
        "docker", "exec", "-i", "novgorod_postgres", "psql",
        "-U", "novgorod-dev", "-d", "novgorod-dev"
    ]
    res = subprocess.run(psql_cmd, input=full_sql, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode != 0:
        print(f"Error executing SQL: {res.stderr}")
        sys.exit(1)
    
    print("Database sync completed successfully!")
    
    # Final count check
    count_res = subprocess.run(
        ["docker", "exec", "novgorod_postgres", "psql", "-U", "novgorod-dev", "-d", "novgorod-dev", "-t", "-A", "-c", "SELECT count(*) FROM product_image;"],
        stdout=subprocess.PIPE, text=True, check=True
    )
    print(f"Total product_image records now in DB: {count_res.stdout.strip()}")

if __name__ == "__main__":
    sync_all()
