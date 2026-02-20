import { useEffect, useRef } from "preact/hooks";
import type { Post } from "../domains/posts.ts";
import type { PostSortParams } from "../services/posts/types.ts";

const SORT_OPTIONS: { label: string; value: PostSortParams }[] = [
  { label: "Newest", value: { sortBy: "CREATED_AT", sortDirection: "DESC" } },
  { label: "Oldest", value: { sortBy: "CREATED_AT", sortDirection: "ASC" } },
  {
    label: "Last updated",
    value: { sortBy: "UPDATED_AT", sortDirection: "DESC" },
  },
  { label: "Title A\u2013Z", value: { sortBy: "TITLE", sortDirection: "ASC" } },
  {
    label: "Title Z\u2013A",
    value: { sortBy: "TITLE", sortDirection: "DESC" },
  },
];

function sortIndex(sort: PostSortParams): number {
  const idx = SORT_OPTIONS.findIndex(
    (o) =>
      o.value.sortBy === sort.sortBy &&
      o.value.sortDirection === sort.sortDirection,
  );
  return idx >= 0 ? idx : 0;
}

interface PostTabsProps {
  posts: Post[];
  activePostId: string | null;
  onSelectPost: (id: string) => void;
  onNewPost: () => void;
  isCreating: boolean;
  onLoadMore: () => void;
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  sort: PostSortParams;
  onSortChange: (sort: PostSortParams) => void;
}

export function PostTabs(
  {
    posts,
    activePostId,
    onSelectPost,
    onNewPost,
    isCreating,
    onLoadMore,
    hasNextPage,
    isFetchingNextPage,
    sort,
    onSortChange,
  }: PostTabsProps,
) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    const root = scrollRef.current;
    if (!sentinel || !root) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
          onLoadMore();
        }
      },
      { root, rootMargin: "0px 0px 200px 0px" },
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [hasNextPage, isFetchingNextPage, onLoadMore]);

  return (
    <div class="w-64 bg-white border-r border-gray-200 flex flex-col flex-shrink-0">
      {/* Header */}
      <div class="p-3 border-b border-gray-200 space-y-2">
        <button
          onClick={onNewPost}
          disabled={isCreating}
          class="w-full px-3 py-2 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 transition-colors"
        >
          {isCreating ? "Creating..." : "+ New Post"}
        </button>
        <select
          value={sortIndex(sort)}
          onChange={(e) => {
            const idx = Number((e.target as HTMLSelectElement).value);
            onSortChange(SORT_OPTIONS[idx].value);
          }}
          class="w-full px-2 py-1.5 text-xs border border-gray-200 rounded-md bg-white text-gray-600 focus:outline-none focus:ring-1 focus:ring-indigo-400"
        >
          {SORT_OPTIONS.map((opt, i) => (
            <option key={i} value={i}>{opt.label}</option>
          ))}
        </select>
      </div>

      {/* Post list */}
      <div ref={scrollRef} class="flex-1 overflow-y-auto">
        {posts.length === 0
          ? (
            <p class="text-sm text-gray-400 p-4 text-center">
              No posts yet
            </p>
          )
          : posts.map((post) => (
            <button
              key={post.id}
              onClick={() => onSelectPost(post.id)}
              class={`w-full text-left px-4 py-3 border-b border-gray-100 hover:bg-gray-50 transition-colors ${
                activePostId === post.id
                  ? "bg-indigo-50 border-l-2 border-l-indigo-500"
                  : ""
              }`}
            >
              <div class="text-sm font-medium text-gray-900 truncate">
                {post.title || "Untitled"}
              </div>
              <div class="flex items-center gap-2 mt-1">
                <span
                  class={`text-xs px-1.5 py-0.5 rounded ${
                    post.isPublished
                      ? "bg-green-100 text-green-700"
                      : "bg-gray-100 text-gray-500"
                  }`}
                >
                  {post.isPublished ? "Published" : "Draft"}
                </span>
                {post.updatedAt && (
                  <span class="text-xs text-gray-400">
                    {new Date(post.updatedAt).toLocaleDateString()}
                  </span>
                )}
              </div>
            </button>
          ))}

        {/* Sentinel for infinite scroll */}
        <div ref={sentinelRef} class="h-1" />

        {isFetchingNextPage && (
          <div class="flex justify-center py-3">
            <div class="w-5 h-5 border-2 border-indigo-300 border-t-indigo-600 rounded-full animate-spin" />
          </div>
        )}
      </div>
    </div>
  );
}
