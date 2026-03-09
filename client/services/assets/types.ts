import { UUID } from "../../domains/common.ts";

export interface AssetUrls {
  thumbnail: string;
  small: string;
  medium: string;
  large: string;
  original: string;
}

export interface Asset {
  id: UUID;
  originalFilename: string;
  mimeType: string;
  sizeBytes: number;
  urls: AssetUrls;
  createdAt: string;
}

export interface UploadResult {
  id: UUID;
  originalFilename: string;
  mimeType: string;
  sizeBytes: number;
  urls: AssetUrls;
  createdAt: string;
}

export interface AssetConnection {
  pageInfo: {
    hasNextPage: boolean;
    endCursor: string | null;
  };
  nodes: Asset[];
}
