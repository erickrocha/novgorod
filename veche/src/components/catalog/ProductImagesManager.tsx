import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  UploadCloud,
  Trash2,
  Star,
  RefreshCw,
  Image as ImageIcon,
  CheckCircle2,
  Clock,
  AlertCircle,
} from "lucide-react";
import Button from "@/components/ui/button/Button";
import Badge from "@/components/ui/badge/Badge";
import {
  productImageService,
  type ProductImage,
} from "@/services/productImageService";

interface ProductImagesManagerProps {
  productId: number;
  readOnly?: boolean;
}

interface UploadProgressItem {
  id: string;
  name: string;
  size: number;
  progress: number;
  status: "uploading" | "processing" | "success" | "error";
  errorMessage?: string;
}

export default function ProductImagesManager({
  productId,
  readOnly = false,
}: ProductImagesManagerProps) {
  const [images, setImages] = useState<ProductImage[]>([]);
  const [loading, setLoading] = useState(false);
  const [uploadQueue, setUploadQueue] = useState<UploadProgressItem[]>([]);
  const [dragActive, setDragActive] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [actionLoadingId, setActionLoadingId] = useState<number | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const fetchImages = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const list = await productImageService.list(productId);
      setImages(list);
    } catch (err: any) {
      setError(err?.response?.data?.message || "Failed to load product images");
    } finally {
      setLoading(false);
    }
  }, [productId]);

  useEffect(() => {
    if (productId) {
      fetchImages();
    }
  }, [productId, fetchImages]);

  // Polling mechanism while any image is in "pending" status (waiting for SQS worker)
  useEffect(() => {
    const hasPending = images.some((img) => img.storageStatus === "pending");
    if (!hasPending) return;

    const interval = setInterval(async () => {
      try {
        const list = await productImageService.list(productId);
        setImages(list);
      } catch {
        // quiet poll
      }
    }, 2500);

    return () => clearInterval(interval);
  }, [images, productId]);

  const handleFiles = async (selectedFiles: FileList | File[]) => {
    const files = Array.from(selectedFiles).filter((f) =>
      f.type.startsWith("image/")
    );
    if (files.length === 0) return;

    const newQueueItems: UploadProgressItem[] = files.map((file) => ({
      id: `${file.name}-${Date.now()}-${Math.random()}`,
      name: file.name,
      size: file.size,
      progress: 0,
      status: "uploading",
    }));

    setUploadQueue((prev) => [...prev, ...newQueueItems]);

    try {
      // 1. Presign batch
      const presignRequests = files.map((f) => ({
        originalFilename: f.name,
        mimeType: f.type || "application/octet-stream",
        sizeBytes: f.size,
      }));

      const presignedItems = await productImageService.presignBatch(
        productId,
        presignRequests
      );

      // 2. Upload each file directly to S3
      await Promise.all(
        presignedItems.map(async (item, index) => {
          const file = files[index];
          const queueItem = newQueueItems[index];

          try {
            await productImageService.uploadFileToS3(item.uploadUrl, file, (percent) => {
              setUploadQueue((prev) =>
                prev.map((q) =>
                  q.id === queueItem.id
                    ? { ...q, progress: percent, status: percent === 100 ? "processing" : "uploading" }
                    : q
                )
              );
            });

            setUploadQueue((prev) =>
              prev.map((q) =>
                q.id === queueItem.id ? { ...q, progress: 100, status: "success" } : q
              )
            );
          } catch (uploadErr: any) {
            setUploadQueue((prev) =>
              prev.map((q) =>
                q.id === queueItem.id
                  ? {
                      ...q,
                      status: "error",
                      errorMessage: uploadErr.message || "Upload to S3 failed",
                    }
                  : q
              )
            );
          }
        })
      );

      // Refresh image list
      await fetchImages();

      // Clear completed queue after 3 seconds
      setTimeout(() => {
        setUploadQueue((prev) => prev.filter((q) => q.status !== "success"));
      }, 3000);
    } catch (batchErr: any) {
      setError(
        batchErr?.response?.data?.message || "Failed to initiate presigned upload"
      );
      setUploadQueue((prev) =>
        prev.map((q) =>
          newQueueItems.some((nq) => nq.id === q.id)
            ? { ...q, status: "error", errorMessage: "Failed to request upload URL" }
            : q
        )
      );
    }
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      handleFiles(e.dataTransfer.files);
    }
  };

  const handleDelete = async (imageId: number) => {
    if (!window.confirm("Are you sure you want to delete this product photo?")) {
      return;
    }
    try {
      setActionLoadingId(imageId);
      await productImageService.delete(productId, imageId);
      setImages((prev) => prev.filter((img) => img.id !== imageId));
    } catch (err: any) {
      setError(err?.response?.data?.message || "Failed to delete image");
    } finally {
      setActionLoadingId(null);
    }
  };

  const handleSetPrimary = async (imageId: number) => {
    try {
      setActionLoadingId(imageId);
      await productImageService.setPrimary(productId, imageId);
      setImages((prev) =>
        prev.map((img) => ({
          ...img,
          isPrimary: img.id === imageId,
        }))
      );
    } catch (err: any) {
      setError(err?.response?.data?.message || "Failed to set primary image");
    } finally {
      setActionLoadingId(null);
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-base font-medium text-gray-800 dark:text-white/90">
            Product Photos
          </h3>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Upload single or multiple images directly to S3 with CloudFront CDN distribution.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          startIcon={<RefreshCw size={14} className={loading ? "animate-spin" : ""} />}
          onClick={fetchImages}
          disabled={loading}
        >
          Refresh
        </Button>
      </div>

      {error && (
        <div className="flex items-center gap-2 rounded-lg border border-error-200 bg-error-50 p-3 text-sm text-error-700 dark:border-error-800 dark:bg-error-900/20 dark:text-error-400">
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {/* Upload Drag & Drop Zone */}
      {!readOnly && (
        <div>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            accept="image/*"
            className="hidden"
            onChange={(e) => {
              if (e.target.files) handleFiles(e.target.files);
              e.target.value = "";
            }}
          />

          <div
            onDragEnter={handleDrag}
            onDragOver={handleDrag}
            onDragLeave={handleDrag}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            className={`flex cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-dashed p-6 transition-all ${
              dragActive
                ? "border-brand-500 bg-brand-50/50 dark:border-brand-400 dark:bg-brand-950/20"
                : "border-gray-300 hover:border-brand-400 dark:border-gray-700 dark:hover:border-brand-500"
            }`}
          >
            <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-brand-50 text-brand-500 dark:bg-brand-900/30 dark:text-brand-400">
              <UploadCloud size={24} />
            </div>
            <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
              <span className="text-brand-500 hover:underline">Click to upload</span> or drag and drop
            </p>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              PNG, JPG, WebP, GIF (Uploads 1 or N images directly to S3)
            </p>
          </div>
        </div>
      )}

      {/* Progress Queue */}
      {uploadQueue.length > 0 && (
        <div className="space-y-2 rounded-xl border border-gray-200 bg-gray-50/50 p-4 dark:border-gray-800 dark:bg-gray-900/50">
          <h4 className="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400">
            Upload Progress ({uploadQueue.length})
          </h4>
          <div className="space-y-2">
            {uploadQueue.map((item) => (
              <div
                key={item.id}
                className="flex items-center justify-between rounded-lg bg-white p-3 shadow-xs dark:bg-gray-800"
              >
                <div className="flex items-center gap-3 overflow-hidden">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-300">
                    <ImageIcon size={16} />
                  </div>
                  <div className="min-w-0">
                    <p className="truncate text-xs font-medium text-gray-800 dark:text-gray-200">
                      {item.name}
                    </p>
                    <p className="text-[11px] text-gray-400">{formatFileSize(item.size)}</p>
                  </div>
                </div>

                <div className="flex items-center gap-4">
                  <div className="w-24">
                    <div className="h-1.5 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
                      <div
                        className={`h-full transition-all duration-300 ${
                          item.status === "error"
                            ? "bg-error-500"
                            : item.status === "processing"
                              ? "bg-amber-500 animate-pulse"
                              : "bg-brand-500"
                        }`}
                        style={{ width: `${item.progress}%` }}
                      />
                    </div>
                  </div>

                  <div className="w-24 text-right">
                    {item.status === "uploading" && (
                      <span className="text-xs font-medium text-brand-600 dark:text-brand-400">
                        {item.progress}%
                      </span>
                    )}
                    {item.status === "processing" && (
                      <span className="flex items-center justify-end gap-1 text-[11px] text-amber-600 dark:text-amber-400">
                        <Clock size={12} className="animate-spin" /> SQS...
                      </span>
                    )}
                    {item.status === "success" && (
                      <span className="flex items-center justify-end gap-1 text-[11px] text-success-600 dark:text-success-400">
                        <CheckCircle2 size={12} /> Ready
                      </span>
                    )}
                    {item.status === "error" && (
                      <span className="text-[11px] text-error-600 dark:text-error-400">Failed</span>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Images Gallery Grid */}
      <div>
        {images.length === 0 && !loading ? (
          <div className="flex flex-col items-center justify-center rounded-xl border border-dashed border-gray-200 py-10 text-center dark:border-gray-800">
            <ImageIcon size={36} className="text-gray-300 dark:text-gray-600" />
            <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">
              No photos added to this product yet.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
            {images.map((img) => {
              const isActionLoading = actionLoadingId === img.id;
              const isPending = img.storageStatus === "pending";

              return (
                <div
                  key={img.id}
                  className={`group relative flex flex-col overflow-hidden rounded-xl border bg-white shadow-xs transition-all dark:bg-gray-800 ${
                    img.isPrimary
                      ? "border-brand-500 ring-2 ring-brand-500/20 dark:border-brand-400"
                      : "border-gray-200 hover:border-gray-300 dark:border-gray-700 dark:hover:border-gray-600"
                  }`}
                >
                  {/* Thumbnail */}
                  <div className="relative aspect-square w-full overflow-hidden bg-gray-100 dark:bg-gray-900">
                    {img.cdnUrl ? (
                      <img
                        src={img.cdnUrl}
                        alt={img.altText || img.originalFilename}
                        className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
                        loading="lazy"
                      />
                    ) : (
                      <div className="flex h-full w-full items-center justify-center text-gray-400">
                        <ImageIcon size={32} />
                      </div>
                    )}

                    {/* Primary Badge */}
                    {img.isPrimary && (
                      <div className="absolute top-2 left-2">
                        <Badge variant="solid" color="primary" size="sm" startIcon={<Star size={10} />}>
                          Primary
                        </Badge>
                      </div>
                    )}

                    {/* SQS Processing Status Badge */}
                    {isPending && (
                      <div className="absolute top-2 right-2">
                        <Badge variant="light" color="warning" size="sm" startIcon={<Clock size={10} className="animate-spin" />}>
                          Processing
                        </Badge>
                      </div>
                    )}
                  </div>

                  {/* Metadata and Actions */}
                  <div className="flex flex-1 flex-col justify-between p-3">
                    <div>
                      <p
                        className="truncate text-xs font-medium text-gray-800 dark:text-gray-200"
                        title={img.originalFilename}
                      >
                        {img.originalFilename}
                      </p>
                      <p className="text-[10px] text-gray-400">
                        {formatFileSize(img.sizeBytes)} • {img.mimeType.split("/")[1]?.toUpperCase() || "IMG"}
                      </p>
                    </div>

                    {!readOnly && (
                      <div className="mt-3 flex items-center justify-between border-t border-gray-100 pt-2 dark:border-gray-700/60">
                        {!img.isPrimary ? (
                          <button
                            type="button"
                            disabled={isActionLoading || isPending}
                            onClick={() => handleSetPrimary(img.id)}
                            className="inline-flex items-center gap-1 text-[11px] font-medium text-gray-500 hover:text-brand-600 dark:text-gray-400 dark:hover:text-brand-400 disabled:opacity-50"
                          >
                            <Star size={12} />
                            Set primary
                          </button>
                        ) : (
                          <span className="text-[11px] font-medium text-brand-600 dark:text-brand-400">
                            Main photo
                          </span>
                        )}

                        <button
                          type="button"
                          disabled={isActionLoading}
                          onClick={() => handleDelete(img.id)}
                          className="text-gray-400 hover:text-error-600 dark:hover:text-error-400 disabled:opacity-50"
                          title="Delete photo"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
