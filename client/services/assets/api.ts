import { getGraphQLClient } from "../../utils/graphql_client.ts";
import type { AssetConnection, UploadResult } from "./types.ts";
import type { UUID } from "../../domains/common.ts";

const GET_ASSETS_QUERY = `
  query GetAssets($after: String, $first: Int) {
    assets(after: $after, first: $first) {
      pageInfo { hasNextPage endCursor }
      nodes {
        id
        originalFilename
        mimeType
        sizeBytes
        createdAt
        urls { thumbnail small medium large original }
      }
    }
  }
`;

const DELETE_ASSET_MUTATION = `
  mutation DeleteAsset($id: UUID!) {
    deleteAsset(id: $id) {
      ... on DeletedAsset { id }
      ... on DbError { message }
      ... on AuthError { message }
    }
  }
`;

export async function uploadAsset(file: File): Promise<UploadResult> {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch("/api/upload", {
    method: "POST",
    body: form,
    credentials: "include",
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? "Upload failed");
  }
  return res.json();
}

export async function getAssets(after?: string): Promise<AssetConnection> {
  const client = getGraphQLClient();
  const data = await client.request<{ assets: AssetConnection }>(
    GET_ASSETS_QUERY,
    { after },
  );
  return data.assets;
}

export async function deleteAsset(id: UUID): Promise<void> {
  const client = getGraphQLClient();
  await client.request(DELETE_ASSET_MUTATION, { id });
}
