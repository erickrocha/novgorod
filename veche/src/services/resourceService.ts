import axios from "axios";
import { api } from "./api";
import type {
  AvatarPresignRequest,
  AvatarPresignResponse,
  Person,
  PersonAddress,
  PersonAddressInput,
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

  async getAddressesByPerson(personId: number): Promise<PersonAddress[]> {
    const res = await api.get<PersonAddress[]>(`/person-addresses/by-person/${personId}`);
    return res.data;
  },

  async addAddress(data: PersonAddressInput): Promise<PersonAddress> {
    const res = await api.post<PersonAddress>("/person-addresses", data);
    return res.data;
  },

  async updateAddress(id: number, data: PersonAddressInput): Promise<PersonAddress> {
    const res = await api.put<PersonAddress>(`/person-addresses/${id}`, data);
    return res.data;
  },

  async deleteAddress(id: number): Promise<void> {
    await api.delete(`/person-addresses/${id}`);
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
