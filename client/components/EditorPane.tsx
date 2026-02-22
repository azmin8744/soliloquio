import type { Post } from "../domains/posts.ts";
import type { EditorBuffer } from "../utils/workspace_signals.ts";

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
}: EditorPaneProps) {
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

  return (
    <div class="flex-1 flex flex-col min-w-0">
      {/* Toolbar */}
      <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-200 bg-white">
        <button
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
          onClick={onSave}
          disabled={isSaving || !isDirty}
          class="px-3 py-1.5 text-sm font-medium text-white bg-indigo-600 rounded-md hover:bg-indigo-700 disabled:opacity-50 transition-colors"
        >
          {isSaving ? "Saving..." : "Save"}
        </button>

        <button
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
          <div class="flex-1 flex flex-col p-4 gap-3 overflow-y-auto bg-gray-50">
            <input
              type="text"
              value={buffer.title}
              onInput={(e) =>
                onBufferChange({
                  title: (e.target as HTMLInputElement).value,
                })}
              placeholder="Post title"
              class="w-full px-3 py-2 text-lg font-semibold border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none"
            />
            <input
              type="text"
              value={buffer.slug}
              onInput={(e) =>
                onBufferChange({
                  slug: (e.target as HTMLInputElement).value,
                })}
              placeholder="slug (auto-generated from title if empty)"
              class="w-full px-3 py-2 text-sm font-mono border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none text-gray-600"
            />
            <textarea
              value={buffer.description}
              onInput={(e) =>
                onBufferChange({
                  description: (e.target as HTMLTextAreaElement).value,
                })}
              rows={2}
              placeholder="Short description…"
              class="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none resize-none text-gray-600"
            />
            <textarea
              value={buffer.markdownContent}
              onInput={(e) =>
                onBufferChange({
                  markdownContent: (e.target as HTMLTextAreaElement).value,
                })}
              placeholder="Write your post in Markdown..."
              class="flex-1 w-full px-3 py-2 font-mono text-sm border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 outline-none resize-none"
            />
          </div>
        )}
    </div>
  );
}
