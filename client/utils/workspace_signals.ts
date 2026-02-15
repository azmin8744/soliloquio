import { computed, signal } from "@preact/signals";
import type { Post } from "../domains/posts.ts";

export interface EditorBuffer {
  title: string;
  markdownContent: string;
  isPublished: boolean;
}

export const postsSignal = signal<Post[]>([]);
export const activePostId = signal<string | null>(null);
export const isPreviewToggled = signal(false);
export const editorBuffer = signal<EditorBuffer>({
  title: "",
  markdownContent: "",
  isPublished: false,
});

// Snapshot of buffer at load time, used to detect changes
export const lastSavedBuffer = signal<EditorBuffer>({
  title: "",
  markdownContent: "",
  isPublished: false,
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
    saved.isPublished !== current.isPublished;
});

/** Load a post into the editor buffer and mark as clean */
export function loadPostIntoBuffer(post: Post) {
  const buf: EditorBuffer = {
    title: post.title,
    markdownContent: post.markdownContent,
    isPublished: post.isPublished,
  };
  editorBuffer.value = buf;
  lastSavedBuffer.value = { ...buf };
}

/** Mark current buffer as saved (after successful save) */
export function markBufferClean() {
  lastSavedBuffer.value = { ...editorBuffer.value };
}
