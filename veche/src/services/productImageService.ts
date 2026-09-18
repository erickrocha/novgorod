import axios from "axios";
import { api } from "./api";

export interface ProductImage {
  id: number;
  uuid: string;
  tenantId?: number | null;
  productId: number;
  skuId?: number | null;
  altText?: string | null;
  sortOrder: number;
  isPrimary: boolean;
  storageProvider: string;
  bucket: string;
  objectKey: string;
  originalFilename: string;
  mimeType: string;
  sizeBytes: number;
  storageStatus: "pending" | "available" | "delete_pending" | "deleted" | "failed";
  cdnUrl?: string | null;
}

export interface PresignImageRequest {
  originalFilename: string;
  mimeType: string;
  sizeBytes: number;
  skuId?: number | null;
  altText?: string | null;
  isPrimary?: boolean;
  sortOrder?: number;
}

export interface PresignedItemResponse {
  uploadUrl: string;
  objectKey: string;
  cdnUrl?: string | null;
  image: ProductImage;
}

export const productImageService = {
  async list(productId: number): Promise<ProductImage[]> {
    const res = await api.get<ProductImage[]>(`/products/${productId}/images`);
    return res.data;
  },

  async presignBatch(
    productId: number,
    images: PresignImageRequest[]
  ): Promise<PresignedItemResponse[]> {
    const res = await api.post<PresignedItemResponse[]>(
      `/products/${productId}/images/presign`,
      { images }
    );
    return res.data;
  },

  async uploadFileToS3(
    uploadUrl: string,
    file: File,
    onProgress?: (progressPercent: number) => void
  ): Promise<void> {
    // Direct S3 upload must not include app authorization headers
    await axios.put(uploadUrl, file, {
      headers: {
        "Content-Type": file.type || "application/octet-stream",
      },
      onUploadProgress: (progressEvent) => {
        if (progressEvent.total && onProgress) {
          const percent = Math.round((progressEvent.loaded * 100) / progressEvent.total);
          onProgress(percent);
        }
      },
    });
  },

  async uploadImages(
    productId: number,
    files: File[],
    onProgress?: (index: number, percent: number) => void
  ): Promise<ProductImage[]> {
    if (files.length === 0) return [];

    const presignRequests: PresignImageRequest[] = files.map((file) => ({
      originalFilename: file.name,
      mimeType: file.type || "application/octet-stream",
      sizeBytes: file.size,
    }));

    const presignedItems = await this.presignBatch(productId, presignRequests);

    // Upload files directly to S3 concurrently or in parallel
    await Promise.all(
      presignedItems.map(async (item, idx) => {
        const file = files[idx];
        if (file) {
          await this.uploadFileToS3(item.uploadUrl, file, (pct) => {
            if (onProgress) onProgress(idx, pct);
          });
        }
      })
    );

    return presignedItems.map((item) => item.image);
  },

  async delete(productId: number, imageId: number): Promise<void> {
    await api.delete(`/products/${productId}/images/${imageId}`);
  },

  async setPrimary(productId: number, imageId: number): Promise<void> {
    await api.put(`/products/${productId}/images/${imageId}/primary`);
  },
};
