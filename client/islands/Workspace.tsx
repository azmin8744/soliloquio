import { useCallback, useEffect, useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { getQueryClient } from "../utils/query_client.ts";
import { useMe, useResendVerificationEmail } from "../services/auth/hooks.ts";
import {
  useCreatePost,
  useDeletePost,
  usePosts,
  useUpdatePost,
} from "../services/posts/hooks.ts";
import {
  activePost,
  activePostId,
  editorBuffer,
  isDirty,
  isMetaPaneOpen,
  isPreviewToggled,
  lastSavedBuffer,
  loadPostIntoBuffer,
  markBufferClean,
  metaPaneTab,
  postsSignal,
  searchSignal,
  setSearch,
  setSort,
  sortSignal,
} from "../utils/workspace_signals.ts";
import type { EditorBuffer, MetaPaneTab } from "../utils/workspace_signals.ts";
import { useAutoSave } from "../utils/use_auto_save.ts";
import { PostTabs } from "../components/PostTabs.tsx";
import { EditorPane } from "../components/EditorPane.tsx";
import { MetadataPane } from "../components/MetadataPane.tsx";
import { EmailVerificationBanner } from "../components/EmailVerificationBanner.tsx";

const RECOVERY_KEY = "soliloquio_editor_recovery";

function WorkspaceInner() {
  const { data: user, isLoading: authLoading } = useMe();
  const currentSort = sortSignal.value;
  const currentSearch = searchSignal.value;
  const {
    data: postsData,
    isLoading: _postsLoading,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = usePosts(currentSort, currentSearch);
  const createPost = useCreatePost();
  const updatePost = useUpdatePost();
  const deletePost = useDeletePost();
  const resendMutation = useResendVerificationEmail();
  const [recoveryToast, setRecoveryToast] = useState(false);

  // Sync posts query → signal
  useEffect(() => {
    if (postsData) {
      postsSignal.value = postsData.pages.flatMap((p) => p.nodes);
    }
  }, [postsData]);

  // Check localStorage recovery on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(RECOVERY_KEY);
      if (saved) setRecoveryToast(true);
    } catch { /* noop */ }
  }, []);

  const restoreBuffer = useCallback(() => {
    try {
      const raw = localStorage.getItem(RECOVERY_KEY);
      if (!raw) return;
      const { postId, buffer } = JSON.parse(raw);
      if (postId && buffer) {
        activePostId.value = postId;
        editorBuffer.value = buffer;
        // Mark dirty so user can save
        lastSavedBuffer.value = {
          title: "",
          markdownContent: "",
          isPublished: false,
          description: "",
          slug: "",
          coverImage: "",
        };
      }
      localStorage.removeItem(RECOVERY_KEY);
    } catch { /* noop */ }
    setRecoveryToast(false);
  }, []);

  const dismissRecovery = useCallback(() => {
    try {
      localStorage.removeItem(RECOVERY_KEY);
    } catch { /* noop */ }
    setRecoveryToast(false);
  }, []);

  // Save handler
  const doSave = useCallback(async () => {
    const post = activePost.value;
    if (!post || !isDirty.value) return;
    const buf = editorBuffer.value;
    try {
      const data = await updatePost.mutateAsync({
        id: post.id,
        title: buf.title,
        content: buf.markdownContent,
        isPublished: buf.isPublished,
        description: buf.description || undefined,
        slug: buf.slug || undefined,
        coverImage: buf.coverImage || undefined,
      });
      if ("id" in data) markBufferClean();
    } catch {
      // error state visible via updatePost.isError
    }
  }, []);

  // Auto-save
  useAutoSave(doSave, isDirty.value);

  // Post switching with dirty guard
  const handleSelectPost = useCallback((id: string) => {
    if (id === activePostId.value) return;
    if (isDirty.value) {
      const choice = confirm(
        "You have unsaved changes. Save before switching?",
      );
      if (choice) {
        doSave();
      }
      // If they cancel, we still switch (discard). The confirm wording
      // is "OK = save, Cancel = discard".
    }
    activePostId.value = id;
    const post = postsSignal.value.find((p) => p.id === id);
    if (post) loadPostIntoBuffer(post);
    isPreviewToggled.value = false;
  }, [doSave]);

  // New post
  const handleNewPost = useCallback(() => {
    createPost.mutate(
      { title: "Untitled", content: "", isPublished: false },
      {
        onSuccess: (data) => {
          if ("id" in data) {
            activePostId.value = data.id;
            loadPostIntoBuffer(data as import("../domains/posts.ts").Post);
          }
        },
      },
    );
  }, []);

  // Delete
  const handleDelete = useCallback(() => {
    const post = activePost.value;
    if (!post) return;
    deletePost.mutate({ id: post.id }, {
      onSuccess: () => {
        activePostId.value = null;
        editorBuffer.value = {
          title: "",
          markdownContent: "",
          isPublished: false,
          description: "",
          slug: "",
          coverImage: "",
        };
        lastSavedBuffer.value = {
          title: "",
          markdownContent: "",
          isPublished: false,
          description: "",
          slug: "",
          coverImage: "",
        };
      },
    });
  }, []);

  // Buffer change
  const handleBufferChange = useCallback((partial: Partial<EditorBuffer>) => {
    editorBuffer.value = { ...editorBuffer.value, ...partial };
  }, []);

  // Meta pane toggle
  const handleToggleMetaTab = useCallback((tab: MetaPaneTab) => {
    if (isMetaPaneOpen.value && metaPaneTab.value === tab) {
      isMetaPaneOpen.value = false;
    } else {
      metaPaneTab.value = tab;
      isMetaPaneOpen.value = true;
    }
  }, []);

  if (authLoading) {
    return (
      <div class="flex-1 flex items-center justify-center text-gray-400">
        Loading...
      </div>
    );
  }

  if (!user) return null;

  return (
    <div class="flex-1 flex flex-col min-h-0 overflow-hidden">
      {!user.emailVerifiedAt && (
        <EmailVerificationBanner resendMutation={resendMutation} />
      )}
      <div class="flex flex-1 min-h-0">
        <PostTabs
          posts={postsSignal.value}
          activePostId={activePostId.value}
          onSelectPost={handleSelectPost}
          onNewPost={handleNewPost}
          isCreating={createPost.isPending}
          onLoadMore={() => fetchNextPage()}
          hasNextPage={!!hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          sort={currentSort}
          onSortChange={setSort}
          search={currentSearch}
          onSearchChange={setSearch}
          emailVerified={!!user.emailVerifiedAt}
        />
        <EditorPane
          editorBuffer={editorBuffer.value}
          isPreviewToggled={isPreviewToggled.value}
          activePost={activePost.value}
          onBufferChange={handleBufferChange}
          onTogglePreview={() =>
            isPreviewToggled.value = !isPreviewToggled.value}
          onTogglePublish={() =>
            handleBufferChange({
              isPublished: !editorBuffer.value.isPublished,
            })}
          onSave={doSave}
          isSaving={updatePost.isPending}
          isDirty={isDirty.value}
          onDelete={handleDelete}
          isDeleting={deletePost.isPending}
          isMetaPaneOpen={isMetaPaneOpen.value}
          activeMetaTab={metaPaneTab.value}
          onToggleMetaTab={handleToggleMetaTab}
        />
        {isMetaPaneOpen.value && (
          <MetadataPane
            activeTab={metaPaneTab.value}
            buffer={editorBuffer.value}
            onBufferChange={handleBufferChange}
            onSwitchToImages={() => handleToggleMetaTab("images")}
          />
        )}
      </div>

      {/* Recovery toast */}
      {recoveryToast && (
        <div class="fixed bottom-4 right-4 bg-white shadow-lg rounded-lg border border-gray-200 p-4 flex items-center gap-3 z-50">
          <span class="text-sm text-gray-700">
            Unsaved work found from previous session.
          </span>
          <button
            type="button"
            onClick={restoreBuffer}
            class="px-3 py-1 text-sm font-medium text-indigo-600 hover:bg-indigo-50 rounded"
          >
            Restore
          </button>
          <button
            type="button"
            onClick={dismissRecovery}
            class="px-3 py-1 text-sm text-gray-500 hover:bg-gray-100 rounded"
          >
            Dismiss
          </button>
        </div>
      )}
    </div>
  );
}

export default function Workspace() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <WorkspaceInner />
    </QueryClientProvider>
  );
}
