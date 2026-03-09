import { useCallback, useEffect, useRef, useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { getQueryClient } from "../utils/query_client.ts";
import {
  useAssets,
  useDeleteAsset,
  useUploadAsset,
} from "../services/assets/hooks.ts";
import type { Asset } from "../services/assets/types.ts";

function CopyIcon() {
  return (
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
      />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        stroke-width="2"
        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
      />
    </svg>
  );
}

function AssetCard({ asset }: { asset: Asset }) {
  const deleteAsset = useDeleteAsset();
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(
      `${globalThis.location.origin}${asset.urls.original}`,
    );
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div class="relative group border rounded-lg overflow-hidden bg-white shadow-sm">
      <img
        src={asset.urls.thumbnail}
        alt={asset.originalFilename}
        class="w-full h-32 object-cover"
        loading="lazy"
      />
      <div class="p-2">
        <p
          class="text-xs text-gray-600 truncate"
          title={asset.originalFilename}
        >
          {asset.originalFilename}
        </p>
        <p class="text-xs text-gray-400">
          {(asset.sizeBytes / 1024).toFixed(1)} KB
        </p>
      </div>
      <div class="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          type="button"
          class="p-1 bg-white rounded shadow text-gray-700 hover:text-indigo-600"
          onClick={handleCopy}
          title={copied ? "Copied!" : "Copy URL"}
        >
          <CopyIcon />
        </button>
        <button
          type="button"
          class="p-1 bg-white rounded shadow text-gray-700 hover:text-red-600"
          onClick={() => deleteAsset.mutate(asset.id)}
          disabled={deleteAsset.isPending}
          title="Delete"
        >
          <TrashIcon />
        </button>
      </div>
    </div>
  );
}

function AssetsLibraryInner() {
  const {
    data,
    isLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useAssets();
  const upload = useUploadAsset();
  const sentinelRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const assets = data?.pages.flatMap((p) => p.nodes) ?? [];

  // Infinite scroll
  useEffect(() => {
    if (!sentinelRef.current) return;
    const obs = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
        fetchNextPage();
      }
    });
    obs.observe(sentinelRef.current);
    return () => obs.disconnect();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  const handleFileChange = useCallback((e: Event) => {
    const files = (e.currentTarget as HTMLInputElement).files;
    if (!files) return;
    for (const file of files) {
      upload.mutate(file);
    }
    if (inputRef.current) inputRef.current.value = "";
  }, []);

  return (
    <div class="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div class="flex items-center justify-between px-6 py-4 border-b bg-white">
        <h1 class="text-lg font-semibold text-gray-900">Assets</h1>
        <label class="cursor-pointer inline-flex items-center gap-2 px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-md hover:bg-indigo-700 transition-colors">
          {upload.isPending ? "Uploading…" : "Upload"}
          <input
            ref={inputRef}
            type="file"
            accept="image/*"
            multiple
            class="hidden"
            onChange={handleFileChange}
            disabled={upload.isPending}
          />
        </label>
      </div>

      {/* Upload error */}
      {upload.isError && (
        <div class="mx-6 mt-3 p-3 text-sm text-red-700 bg-red-50 rounded-md">
          {(upload.error as Error).message}
        </div>
      )}

      {/* Grid */}
      <div class="flex-1 overflow-y-auto p-6">
        {isLoading
          ? <p class="text-sm text-gray-500">Loading assets…</p>
          : assets.length === 0
          ? (
            <div class="flex flex-col items-center justify-center h-48 text-gray-400">
              <p class="text-sm">No assets yet. Upload your first image.</p>
            </div>
          )
          : (
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
              {assets.map((asset) => (
                <AssetCard
                  key={asset.id}
                  asset={asset}
                />
              ))}
            </div>
          )}
        <div ref={sentinelRef} class="h-4" />
        {isFetchingNextPage && (
          <p class="text-center text-sm text-gray-400 mt-2">Loading more…</p>
        )}
      </div>
    </div>
  );
}

export default function AssetsLibrary() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <AssetsLibraryInner />
    </QueryClientProvider>
  );
}
