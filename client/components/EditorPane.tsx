import { useEffect, useRef } from "preact/hooks";
import type { Post } from "../domains/posts.ts";
import type { EditorBuffer, MetaPaneTab } from "../utils/workspace_signals.ts";
import { pendingInsertText } from "../utils/workspace_signals.ts";

interface EditorPaneProps {
  editorBuffer: EditorBuffer;
  isPreviewToggled: boolean;
  activePost: Post | null;
  onBufferChange: (partial: Partial<EditorBuffer>) => void;
  onTogglePreview: () => void;
  onTogglePublish: () => void;
  onSave: () => void;
  isSaving: boolean;
  isDirty: boolean;
  onDelete: () => void;
  isDeleting: boolean;
  isMetaPaneOpen: boolean;
  activeMetaTab: MetaPaneTab;
  onToggleMetaTab: (tab: MetaPaneTab) => void;
}

export function EditorPane({
  editorBuffer: buffer,
  isPreviewToggled,
  activePost,
  onBufferChange,
  onTogglePreview,
  onTogglePublish,
  onSave,
  isSaving,
  isDirty,
  onDelete,
  isDeleting,
  isMetaPaneOpen,
  activeMetaTab,
  onToggleMetaTab,
}: EditorPaneProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Watch pendingInsertText signal and insert at cursor
  useEffect(() => {
    const dispose = pendingInsertText.subscribe((text) => {
      if (text === null) return;
      const el = textareaRef.current;
      if (!el) return;
      const start = el.selectionStart ?? 0;
      const end = el.selectionEnd ?? 0;
      const before = buffer.markdownContent.slice(0, start);
      const after = buffer.markdownContent.slice(end);
      onBufferChange({ markdownContent: before + text + after });
      pendingInsertText.value = null;
      const newCursor = start + text.length;
      requestAnimationFrame(() => {
        el.selectionStart = newCursor;
        el.selectionEnd = newCursor;
        el.focus();
      });
    });
    return dispose;
  }, [buffer.markdownContent, onBufferChange]);

  if (!activePost) {
    return (
      <div class="flex-1 flex items-center justify-center text-gray-400">
        <p>Select a post or create a new one</p>
      </div>
    );
  }

  const handleDelete = () => {
    if (confirm("Delete this post? This cannot be undone.")) {
      onDelete();
    }
  };

  const metaActive = isMetaPaneOpen && activeMetaTab === "meta";
  const imgActive = isMetaPaneOpen && activeMetaTab === "images";

  return (
    <div class="flex-1 flex flex-col min-w-0">
      {/* Toolbar */}
      <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-200 bg-white">
        <button
          type="button"
          onClick={onTogglePreview}
          class={`px-3 py-1.5 text-sm rounded-md transition-colors ${
            isPreviewToggled
              ? "bg-indigo-100 text-indigo-700"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          {isPreviewToggled ? "Edit" : "Preview"}
        </button>

        <label class="flex items-center gap-1.5 text-sm text-gray-600 cursor-pointer ml-2">
          <input
            type="checkbox"
            checked={buffer.isPublished}
            onChange={onTogglePublish}
            class="rounded border-gray-300"
          />
          Published
        </label>

        <div class="flex-1" />

        {isDirty && <span class="text-xs text-amber-600">Unsaved changes</span>}

        <button
          type="button"
          onClick={onSave}
          disabled={isSaving || !isDirty}
          class="px-3 py-1.5 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 transition-colors"
        >
          {isSaving ? "Saving..." : "Save"}
        </button>

        <button
          type="button"
          onClick={handleDelete}
          disabled={isDeleting}
          class="px-3 py-1.5 text-sm font-medium text-red-600 hover:bg-red-50 rounded-md disabled:opacity-50 transition-colors"
        >
          {isDeleting ? "Deleting..." : "Delete"}
        </button>
      </div>

      {/* Editor / Preview */}
      {isPreviewToggled
        ? (
          <div class="flex-1 overflow-y-auto p-6 bg-white">
            <h1 class="text-2xl font-bold mb-4">{buffer.title}</h1>
            <div
              class="prose max-w-none"
              dangerouslySetInnerHTML={{ __html: activePost.content }}
            />
          </div>
        )
        : (
          <div class="flex-1 flex flex-col overflow-y-auto">
            {/* Title */}
            <input
              type="text"
              value={buffer.title}
              onInput={(e) =>
                onBufferChange({
                  title: (e.target as HTMLInputElement).value,
                })}
              placeholder="Post title"
              class="w-full px-4 pt-4 pb-3 text-lg font-semibold border-b border-gray-200 outline-none bg-white placeholder-gray-300"
            />
            {/* Body + icon strip */}
            <div class="relative flex-1 flex flex-col">
              {/* Floating icon strip — scoped to body, below divider */}
              <div class="absolute right-0 top-0 h-full flex flex-col items-center pt-3 gap-2 z-10 w-11">
                {/* Metadata icon */}
                <button
                  type="button"
                  title="Metadata"
                  onClick={() => onToggleMetaTab("meta")}
                  class={`p-1.5 rounded transition-colors ${
                    metaActive
                      ? "text-indigo-600 bg-indigo-50"
                      : "text-gray-400 hover:text-gray-600 hover:bg-gray-100"
                  }`}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M7 7h.01M7 12h.01M7 17h.01M11 7h6M11 12h6M11 17h6"
                    />
                  </svg>
                </button>
                {/* Images icon */}
                <button
                  type="button"
                  title="Images"
                  onClick={() => onToggleMetaTab("images")}
                  class={`p-1.5 rounded transition-colors ${
                    imgActive
                      ? "text-indigo-600 bg-indigo-50"
                      : "text-gray-400 hover:text-gray-600 hover:bg-gray-100"
                  }`}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14M6 6h.01M6 4h12a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2z"
                    />
                  </svg>
                </button>
              </div>

              {/* Body */}
              <textarea
                ref={textareaRef}
                value={buffer.markdownContent}
                onInput={(e) =>
                  onBufferChange({
                    markdownContent: (e.target as HTMLTextAreaElement).value,
                  })}
                placeholder="Write your post in Markdown..."
                class="flex-1 w-full pr-11 px-4 py-3 font-mono text-sm outline-none resize-none bg-white"
              />
            </div>
          </div>
        )}
    </div>
  );
}
