import { computed, signal } from "@preact/signals";
import type { Post } from "../domains/posts.ts";
import type { PostSortParams } from "../services/posts/types.ts";

export interface EditorBuffer {
  title: string;
  markdownContent: string;
  isPublished: boolean;
  description: string;
  slug: string;
}

const SORT_STORAGE_KEY = "soliloquio_sort";
const DEFAULT_SORT: PostSortParams = {
  sortBy: "CREATED_AT",
  sortDirection: "DESC",
};

function loadSort(): PostSortParams {
  try {
    const raw = localStorage.getItem(SORT_STORAGE_KEY);
    if (raw) return JSON.parse(raw);
  } catch { /* noop */ }
  return DEFAULT_SORT;
}

export const sortSignal = signal<PostSortParams>(
  typeof localStorage !== "undefined" ? loadSort() : DEFAULT_SORT,
);

export function setSort(sort: PostSortParams) {
  sortSignal.value = sort;
  try {
    localStorage.setItem(SORT_STORAGE_KEY, JSON.stringify(sort));
  } catch { /* noop */ }
}

export const searchSignal = signal("");
export function setSearch(q: string) {
  searchSignal.value = q;
}

export const postsSignal = signal<Post[]>([]);
export const activePostId = signal<string | null>(null);
export const isPreviewToggled = signal(false);
export const editorBuffer = signal<EditorBuffer>({
  title: "",
  markdownContent: "",
  isPublished: false,
  description: "",
  slug: "",
});

// Snapshot of buffer at load time, used to detect changes
export const lastSavedBuffer = signal<EditorBuffer>({
  title: "",
  markdownContent: "",
  isPublished: false,
  description: "",
  slug: "",
});

export const activePost = computed(() => {
  const id = activePostId.value;
  if (!id) return null;
  return postsSignal.value.find((p) => p.id === id) ?? null;
});

export const isDirty = computed(() => {
  const saved = lastSavedBuffer.value;
  const current = editorBuffer.value;
  return saved.title !== current.title ||
    saved.markdownContent !== current.markdownContent ||
    saved.isPublished !== current.isPublished ||
    saved.description !== current.description ||
    saved.slug !== current.slug;
});

/** Load a post into the editor buffer and mark as clean */
export function loadPostIntoBuffer(post: Post) {
  const buf: EditorBuffer = {
    title: post.title,
    markdownContent: post.markdownContent,
    isPublished: post.isPublished,
    description: post.description ?? "",
    slug: post.slug ?? "",
  };
  editorBuffer.value = buf;
  lastSavedBuffer.value = { ...buf };
}

/** Mark current buffer as saved (after successful save) */
export function markBufferClean() {
  lastSavedBuffer.value = { ...editorBuffer.value };
}
