import axios from "axios";
import { api } from "./api";
import type {
  AvatarPresignRequest,
  AvatarPresignResponse,
  Person,
  PersonInput,
  ResourceProfile,
} from "./types";

export const resourceService = {
  async getProfile(): Promise<ResourceProfile> {
    const res = await api.get<ResourceProfile>("/resource/profile");
    return res.data;
  },

  async updateProfile(data: PersonInput): Promise<Person> {
    const res = await api.put<Person>("/resource/profile", data);
    return res.data;
  },

  async presignAvatar(req: AvatarPresignRequest): Promise<AvatarPresignResponse> {
    const res = await api.post<AvatarPresignResponse>("/resource/avatar/presign", req);
    return res.data;
  },

  async uploadAvatar(
    file: File,
    onProgress?: (progressPercent: number) => void
  ): Promise<{ objectKey: string; cdnUrl?: string | null }> {
    const presignRes = await this.presignAvatar({
      originalFilename: file.name,
      mimeType: file.type || "application/octet-stream",
      sizeBytes: file.size,
    });

    await axios.put(presignRes.uploadUrl, file, {
      headers: {
        "Content-Type": file.type || "application/octet-stream",
      },
      onUploadProgress: (progressEvent) => {
        if (progressEvent.total && onProgress) {
          const percent = Math.round(
            (progressEvent.loaded * 100) / progressEvent.total
          );
          onProgress(percent);
        }
      },
    });

    return {
      objectKey: presignRes.objectKey,
      cdnUrl: presignRes.cdnUrl,
    };
  },
};
