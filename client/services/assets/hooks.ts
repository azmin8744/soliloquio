import {
  useInfiniteQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { deleteAsset, getAssets, uploadAsset } from "./api.ts";
import { assetKeys } from "./keys.ts";
import type { AssetConnection } from "./types.ts";
import type { UUID } from "../../domains/common.ts";

export function useAssets() {
  return useInfiniteQuery<AssetConnection>({
    queryKey: assetKeys.lists(),
    queryFn: ({ pageParam }) => getAssets(pageParam as string | undefined),
    initialPageParam: undefined,
    getNextPageParam: (last) =>
      last.pageInfo.hasNextPage
        ? (last.pageInfo.endCursor ?? undefined)
        : undefined,
  });
}

export function useUploadAsset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (file: File) => uploadAsset(file),
    onSuccess: () => qc.invalidateQueries({ queryKey: assetKeys.lists() }),
  });
}

export function useDeleteAsset() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: UUID) => deleteAsset(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: assetKeys.lists() }),
  });
}
