#!/usr/bin/env python3
"""
download_wine_images.py
Downloads 3 authentic, high-quality images from the web (fabricant catalogues,
winery packshots, bottle photography) for all 50 products in the Novgorod wine catalog.
Saves them under data/wines/images/<product_key>/image_{1,2,3}.jpg
and generates data/wines/images/images_manifest.json with complete metadata.
"""

import os
import sys
import re
import io
import time
import json
import hashlib
import urllib.request
import urllib.parse
from concurrent.futures import ThreadPoolExecutor, as_completed
from PIL import Image

WORKSPACE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DATA_DIR = os.path.join(WORKSPACE_DIR, "data", "wines")
IMAGES_DIR = os.path.join(DATA_DIR, "images")
CATALOG_MJS = os.path.join(WORKSPACE_DIR, "scripts", "generate_wine_catalog.mjs")

USER_AGENT = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"

def load_catalog_wines():
    """Extracts the 50 wine definitions from generate_wine_catalog.mjs."""
    with open(CATALOG_MJS, "r", encoding="utf-8") as f:
        content = f.read()
    
    # We parse the objects from the realWines array
    wine_blocks = re.findall(r'\{\s*key:\s*"([^"]+)",\s*name:\s*"([^"]+)",\s*slug:\s*"([^"]+)"[\s\S]*?brand:\s*"([^"]+)"[\s\S]*?country:\s*"([^"]+)"[\s\S]*?vintage:\s*"([^"]+)"', content)
    
    wines = []
    for idx, (key, name, slug, brand, country, vintage) in enumerate(wine_blocks):
        wines.append({
            "id": idx + 1,
            "key": key,
            "name": name,
            "slug": slug,
            "brand": brand,
            "country": country,
            "vintage": vintage,
        })
    return wines

def search_bing_images(query, max_results=20):
    """Searches Bing Images for direct media URLs."""
    url = f"https://www.bing.com/images/search?q={urllib.parse.quote(query)}&form=HDRSC2&first=1"
    headers = {"User-Agent": USER_AGENT}
    req = urllib.request.Request(url, headers=headers)
    
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            html = resp.read().decode("utf-8", errors="ignore")
            # Extract high-res image URLs embedded in murl attribute
            murls = re.findall(r'&quot;murl&quot;:&quot;(https?://[^&]+)&quot;', html)
            if not murls:
                murls = re.findall(r'\"murl\":\"(https?://[^\"]+)\"', html)
            return murls[:max_results]
    except Exception as e:
        print(f"  [Search Error for '{query}']: {e}")
        return []

def download_and_validate_image(url, timeout=10):
    """Downloads an image from URL and validates it with PIL."""
    headers = {
        "User-Agent": USER_AGENT,
        "Accept": "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        "Referer": "https://www.bing.com/",
    }
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            if resp.status != 200:
                return None
            data = resp.read()
            if len(data) < 4096: # Minimum 4KB
                return None
            
            # Verify using PIL
            img_io = io.BytesIO(data)
            img = Image.open(img_io)
            width, height = img.size
            if width < 200 or height < 200: # Exclude tiny icons
                return None
            
            img.seek(0)
            img_format = img.format.lower() if img.format else "jpeg"
            return {
                "raw_bytes": data,
                "width": width,
                "height": height,
                "format": img_format,
                "url": url,
            }
    except Exception:
        return None

def process_product(wine):
    """Downloads 3 distinct valid images for a given wine product."""
    key = wine["key"]
    name = wine["name"]
    brand = wine["brand"]
    wine_id = wine["id"]
    
    product_dir = os.path.join(IMAGES_DIR, key)
    os.makedirs(product_dir, exist_ok=True)
    
    # Check if 3 valid images already exist
    existing_images = []
    for i in range(1, 4):
        jpg_path = os.path.join(product_dir, f"image_{i}.jpg")
        if os.path.exists(jpg_path) and os.path.getsize(jpg_path) > 4096:
            try:
                with Image.open(jpg_path) as im:
                    w, h = im.size
                    existing_images.append({
                        "sort_order": i - 1,
                        "is_primary": i == 1,
                        "file_name": f"image_{i}.jpg",
                        "path": jpg_path,
                        "width": w,
                        "height": h,
                        "size_bytes": os.path.getsize(jpg_path),
                        "url": "cached_local",
                    })
            except Exception:
                pass
    
    if len(existing_images) == 3:
        print(f"[{wine_id}/50] {name}: 3 images already present locally.")
        return {
            "product_id": wine_id,
            "product_key": key,
            "product_name": name,
            "brand": brand,
            "images": existing_images,
        }
    
    print(f"[{wine_id}/50] Fetching images for: {name} ({brand})...")
    
    # Query variations
    queries = [
        f"{name} bottle packshot",
        f"{brand} {name} vinho",
        f"{name} winery catalogue",
        f"{name} bottle",
    ]
    
    candidate_urls = []
    seen_urls = set()
    for q in queries:
        urls = search_bing_images(q)
        for u in urls:
            if u not in seen_urls and not any(ext in u.lower() for ext in [".svg", ".gif", ".ico"]):
                seen_urls.add(u)
                candidate_urls.append(u)
        if len(candidate_urls) >= 15:
            break
        time.sleep(0.3)
    
    saved_images = list(existing_images)
    seen_hashes = set()
    
    for url in candidate_urls:
        if len(saved_images) >= 3:
            break
        img_info = download_and_validate_image(url)
        if not img_info:
            continue
        
        # Check hash to avoid duplicate images
        sha = hashlib.sha256(img_info["raw_bytes"]).hexdigest()
        if sha in seen_hashes:
            continue
        seen_hashes.add(sha)
        
        idx = len(saved_images) + 1
        file_name = f"image_{idx}.jpg"
        out_path = os.path.join(product_dir, file_name)
        
        try:
            # Convert RGBA/P to RGB for uniform JPEG saving
            pil_img = Image.open(io.BytesIO(img_info["raw_bytes"]))
            if pil_img.mode in ("RGBA", "P", "LA"):
                pil_img = pil_img.convert("RGB")
            pil_img.save(out_path, "JPEG", quality=92, optimize=True)
            
            saved_images.append({
                "sort_order": idx - 1,
                "is_primary": idx == 1,
                "file_name": file_name,
                "path": out_path,
                "width": img_info["width"],
                "height": img_info["height"],
                "size_bytes": os.path.getsize(out_path),
                "url": url,
                "sha256": sha,
            })
            print(f"  -> Saved {file_name} for {name} ({img_info['width']}x{img_info['height']}, {os.path.getsize(out_path)//1024} KB)")
        except Exception as e:
            print(f"  -> Error saving image {idx}: {e}")
    
    # If fewer than 3 were saved, try fallback queries
    if len(saved_images) < 3:
        print(f"  [Warning] Only {len(saved_images)} images saved for {name}, trying fallback queries...")
        fallback_queries = [f"{brand} wine bottle", f"{name}"]
        for fq in fallback_queries:
            if len(saved_images) >= 3:
                break
            for u in search_bing_images(fq):
                if u in seen_urls:
                    continue
                seen_urls.add(u)
                img_info = download_and_validate_image(u)
                if not img_info:
                    continue
                sha = hashlib.sha256(img_info["raw_bytes"]).hexdigest()
                if sha in seen_hashes:
                    continue
                seen_hashes.add(sha)
                idx = len(saved_images) + 1
                file_name = f"image_{idx}.jpg"
                out_path = os.path.join(product_dir, file_name)
                try:
                    pil_img = Image.open(io.BytesIO(img_info["raw_bytes"]))
                    if pil_img.mode in ("RGBA", "P", "LA"):
                        pil_img = pil_img.convert("RGB")
                    pil_img.save(out_path, "JPEG", quality=92, optimize=True)
                    saved_images.append({
                        "sort_order": idx - 1,
                        "is_primary": idx == 1,
                        "file_name": file_name,
                        "path": out_path,
                        "width": img_info["width"],
                        "height": img_info["height"],
                        "size_bytes": os.path.getsize(out_path),
                        "url": u,
                        "sha256": sha,
                    })
                    print(f"  -> [Fallback] Saved {file_name} ({img_info['width']}x{img_info['height']})")
                except Exception:
                    pass
                if len(saved_images) >= 3:
                    break
    
    return {
        "product_id": wine_id,
        "product_key": key,
        "product_name": name,
        "brand": brand,
        "images": saved_images,
    }

def main():
    os.makedirs(IMAGES_DIR, exist_ok=True)
    wines = load_catalog_wines()
    print(f"Starting image download for {len(wines)} wine products...")
    
    manifest_results = []
    
    # Run with a thread pool of 5 workers
    with ThreadPoolExecutor(max_workers=5) as executor:
        futures = {executor.submit(process_product, wine): wine for wine in wines}
        for future in as_completed(futures):
            wine = futures[future]
            try:
                res = future.result()
                manifest_results.append(res)
            except Exception as e:
                print(f"Error processing {wine['name']}: {e}")
    
    # Sort results by product_id
    manifest_results.sort(key=lambda x: x["product_id"])
    
    manifest_path = os.path.join(IMAGES_DIR, "images_manifest.json")
    with open(manifest_path, "w", encoding="utf-8") as f:
        json.dump(manifest_results, f, indent=2, ensure_ascii=False)
    
    total_downloaded = sum(len(item["images"]) for item in manifest_results)
    print(f"\n==========================================")
    print(f"Completed! Downloaded {total_downloaded} total images for {len(manifest_results)} products.")
    print(f"Manifest written to: {manifest_path}")
    print(f"==========================================")

if __name__ == "__main__":
    main()
