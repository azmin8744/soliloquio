# Soliloquio Master UI Specification

## 1. System Overview

* **Name:** Soliloquio (Authentic CMS)
* **Framework:** Fresh (Deno)
* **Data Layer:** GraphQL via a BFF handler (`/api/graphql`) 
* **State Management:** Preact Signals (Global and Island-level)
* **Authentication:** Email/Password via JWT and Refresh Tokens 



---

## 2. Layout Structure (The Three-Column Workspace)

The application uses a fixed-height, full-viewport layout divided into three vertical sections.

### 2.1 Left Column: Global Navigation Rail

* **Width:** Fixed narrow (60-80px).
* **Top Section:** Application Logo.
* **Bottom Section:** * **User Profile:** Displays the current user's email fetched via the `me` query.
    * **Logout Button:** Triggers the `logout` mutation with the current `refreshToken`. On success, redirect to `/login`.





### 2.2 Center Column: Vertical Post Tabs

* **Header:** "New Post" button (triggers `addPost` mutation).
* **List Area:** A scrollable list of post titles from `postsSignal`.
* **Behavior:** * Clicking a title updates `activePostId`.
    * **Guard:** If `isDirty.value` is true, the UI must prompt the user or auto-save via `updatePost` before switching.





### 2.3 Right Column: Workspace

* **Header:**
    * **Toggle Preview:** Switch between "Editor Only" and "Split View."
    * **Publish Toggle:** Binds to `isPublished` Boolean.
* **Pane A (Markdown Editor):** Monospaced textarea binding to `editorBuffer` (`title` and `markdownContent`). 
* **Pane B (Live Preview):** Toggled pane rendering the `content` (HTML) field from the backend.



---

## 3. State Management (TypeScript Signals)

```typescript
import { signal, computed } from "@preact/signals";

[cite_start]/** Defined based on GraphQL Schema [cite: 6] */
export interface Post {
  id: string; [cite_start]// UUID [cite: 3, 10]
  title: string;
  isPublished: boolean;
  markdownContent: string;
  content: string; // HTML
  updatedAt?: string; [cite_start]// NaiveDateTime [cite: 5]
}

export const postsSignal = signal<Post[]>([]);
export const activePostId = signal<string | null>(null);
export const isPreviewToggled = signal<boolean>(true);

[cite_start]// Buffer for unsaved changes [cite: 1, 11]
export const editorBuffer = signal<{
  title: string;
  markdownContent: string;
} | null>(null);

export const activePost = computed(() => {
  const id = activePostId.value;
  return id ? postsSignal.value.find((p) => p.id === id) : null;
});

export const isDirty = computed(() => {
  if (!activePost.value || !editorBuffer.value) return false;
  return (
    editorBuffer.value.title !== activePost.value.title ||
    editorBuffer.value.markdownContent !== activePost.value.markdownContent
  );
});

```

---

## 4. Session Lifecycle & Routing

### 4.1 Unauthenticated Access

* **Path:** Any protected route (e.g., `/`).
* **Logic:** If the `me` query or initial data fetch returns an `AuthError` or `401`, the BFF will attempt a `refreshAccessToken`.
* **Redirect:** If refresh fails, the client must redirect to `/login`.

### 4.2 Login Success

* **Action:** User submits `signIn`.
* **BFF Role:** Sets secure cookies for `access_token` and `refresh_token`.
* **Flow:** 1. Redirect to `/`.
2. Populate `postsSignal` via the `posts` query.
3. Initialize the UI with an empty workspace until a post is selected.



### 4.3 Session Expiration during Editing

* **Trigger:** Backend returns "Token expired" after failed refresh attempts.
* **Action:** Show a "Session Expired" notification. Redirect to `/login`. The implementation should attempt to save `editorBuffer` to `localStorage` as a fallback.

---

## 5. Mutation Definitions (Reference)

Available mutations are listed in @schema.graphql.