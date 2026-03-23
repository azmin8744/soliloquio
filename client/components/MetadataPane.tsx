import { useState } from "preact/hooks";
import type { EditorBuffer, MetaPaneTab } from "../utils/workspace_signals.ts";
import { pendingInsertText } from "../utils/workspace_signals.ts";
import { useAssets, useUploadAsset } from "../services/assets/hooks.ts";
import type { Asset, AssetUrls } from "../services/assets/types.ts";

interface MetadataPaneProps {
  activeTab: MetaPaneTab;
  buffer: EditorBuffer;
  onBufferChange: (partial: Partial<EditorBuffer>) => void;
  onSwitchToImages: () => void;
}

type SizeKey = keyof AssetUrls;
const SIZE_LABELS: { key: SizeKey; label: string }[] = [
  { key: "thumbnail", label: "Thumbnail" },
  { key: "small", label: "Small" },
  { key: "medium", label: "Medium" },
  { key: "large", label: "Large" },
  { key: "original", label: "Original" },
];

interface ImageTileProps {
  asset: Asset;
  isCover: boolean;
  onSetCover: (url: string) => void;
  onClearCover: () => void;
}

function ImageTile({ asset, isCover, onSetCover, onClearCover }: ImageTileProps) {
  const [showSizes, setShowSizes] = useState(false);

  const handleInsert = (sizeKey: SizeKey) => {
    const url = asset.urls[sizeKey];
    const name = asset.originalFilename.replace(/\.[^.]+$/, "");
    pendingInsertText.value = `![${name}](${url})`;
    setShowSizes(false);
  };

  return (
    <div class="relative aspect-square bg-gray-100 rounded overflow-hidden group">
      <img
        src={asset.urls.thumbnail}
        alt={asset.originalFilename}
        class="w-full h-full object-cover"
      />
      {/* Cover badge */}
      {isCover && (
        <div class="absolute top-1 left-1 px-1.5 py-0.5 text-xs font-semibold bg-indigo-600 text-white rounded">
          Cover
        </div>
      )}
      {/* Hover overlay */}
      <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-1">
        <button
          type="button"
          onClick={() => setShowSizes(true)}
          class="px-2 py-1 text-xs font-medium text-white bg-indigo-600 rounded hover:bg-indigo-700"
        >
          Insert
        </button>
        {isCover
          ? (
            <button
              type="button"
              onClick={onClearCover}
              class="px-2 py-1 text-xs font-medium text-white bg-gray-600 rounded hover:bg-gray-700"
            >
              Remove
            </button>
          )
          : (
            <button
              type="button"
              onClick={() => onSetCover(`${globalThis.location.origin}${asset.urls.large}`)}
              class="px-2 py-1 text-xs font-medium text-white bg-emerald-600 rounded hover:bg-emerald-700"
            >
              Set cover
            </button>
          )}
      </div>
      {/* Size dropdown */}
      {showSizes && (
        <div class="absolute inset-0 bg-white flex flex-col z-10">
          <div class="flex items-center justify-between px-2 py-1 border-b border-gray-200">
            <span class="text-xs font-medium text-gray-700">Pick size</span>
            <button
              type="button"
              onClick={() => setShowSizes(false)}
              class="text-gray-400 hover:text-gray-600 text-xs"
            >
              ✕
            </button>
          </div>
          {SIZE_LABELS.map(({ key, label }) => (
            <button
              type="button"
              key={key}
              onClick={() => handleInsert(key)}
              class="text-left px-2 py-1 text-xs hover:bg-indigo-50 hover:text-indigo-700 text-gray-700"
            >
              {label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function MetaTab(
  { buffer, onBufferChange, onSwitchToImages }: Pick<
    MetadataPaneProps,
    "buffer" | "onBufferChange" | "onSwitchToImages"
  >,
) {
  return (
    <div class="flex flex-col gap-3 p-4">
      <div>
        <label class="block text-xs font-medium text-gray-500 mb-1">
          Cover image
        </label>
        {buffer.coverImage
          ? (
            <div class="relative rounded overflow-hidden bg-gray-100">
              <img
                src={buffer.coverImage}
                alt="Cover"
                class="w-full h-24 object-cover"
              />
              <button
                type="button"
                onClick={() => onBufferChange({ coverImage: "" })}
                class="absolute top-1 right-1 px-1.5 py-0.5 text-xs font-medium bg-black/60 text-white rounded hover:bg-black/80"
              >
                Remove
              </button>
            </div>
          )
          : (
            <button
              type="button"
              onClick={onSwitchToImages}
              class="w-full px-2 py-2 text-xs text-indigo-600 border border-dashed border-indigo-300 rounded hover:bg-indigo-50 text-center"
            >
              Pick from Images →
            </button>
          )}
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-500 mb-1">Slug</label>
        <input
          type="text"
          value={buffer.slug}
          onInput={(e) =>
            onBufferChange({ slug: (e.target as HTMLInputElement).value })}
          placeholder="auto-generated from title"
          class="w-full px-2 py-1.5 text-sm font-mono border border-gray-200 rounded outline-none focus:border-indigo-400 text-gray-700"
        />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-500 mb-1">
          Description
        </label>
        <textarea
          value={buffer.description}
          onInput={(e) =>
            onBufferChange({
              description: (e.target as HTMLTextAreaElement).value,
            })}
          rows={2}
          placeholder="Short description…"
          class="w-full px-2 py-1.5 text-sm border border-gray-200 rounded outline-none focus:border-indigo-400 resize-none text-gray-700"
        />
      </div>
    </div>
  );
}

function ImagesTab(
  { buffer, onBufferChange }: Pick<MetadataPaneProps, "buffer" | "onBufferChange">,
) {
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useAssets();
  const upload = useUploadAsset();
  const assets = data?.pages.flatMap((p) => p.nodes) ?? [];

  const handleUpload = (e: Event) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) upload.mutate(file);
  };

  return (
    <div class="flex flex-col flex-1 min-h-0">
      <div class="flex items-center justify-between px-4 py-2 border-b border-gray-200">
        <span class="text-xs font-medium text-gray-700">Images</span>
        <label class="cursor-pointer px-2 py-1 text-xs font-medium text-indigo-600 hover:bg-indigo-50 rounded">
          {upload.isPending ? "Uploading…" : "Upload"}
          <input
            type="file"
            accept="image/*"
            class="hidden"
            onChange={handleUpload}
          />
        </label>
      </div>
      <div class="flex-1 overflow-y-auto p-3">
        <div class="grid grid-cols-2 gap-2">
          {assets.map((asset) => (
            <ImageTile
              key={asset.id}
              asset={asset}
              isCover={!!buffer.coverImage && buffer.coverImage === `${globalThis.location.origin}${asset.urls.large}`}
              onSetCover={(url) => onBufferChange({ coverImage: url })}
              onClearCover={() => onBufferChange({ coverImage: "" })}
            />
          ))}
        </div>
        {hasNextPage && (
          <button
            type="button"
            onClick={() => fetchNextPage()}
            disabled={isFetchingNextPage}
            class="w-full mt-3 py-1.5 text-xs text-gray-500 hover:text-gray-700 disabled:opacity-50"
          >
            {isFetchingNextPage ? "Loading…" : "Load more"}
          </button>
        )}
        {assets.length === 0 && !isFetchingNextPage && (
          <p class="text-xs text-gray-400 text-center py-8">No images yet</p>
        )}
      </div>
    </div>
  );
}

export function MetadataPane(
  { activeTab, buffer, onBufferChange, onSwitchToImages }: MetadataPaneProps,
) {
  return (
    <div class="w-72 border-l border-gray-200 bg-white flex flex-col overflow-y-auto">
      {activeTab === "meta"
        ? (
          <MetaTab
            buffer={buffer}
            onBufferChange={onBufferChange}
            onSwitchToImages={onSwitchToImages}
          />
        )
        : <ImagesTab buffer={buffer} onBufferChange={onBufferChange} />}
    </div>
  );
}
