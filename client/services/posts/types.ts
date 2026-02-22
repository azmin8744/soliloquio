import { UUID } from "../../domains/common.ts";
import { DeletedPost, Post } from "../../domains/posts.ts";
import { AuthError, DbError } from "../../domains/common.ts";

export interface PageInfo {
  hasNextPage: boolean;
  endCursor: string | null;
}

export interface PostConnection {
  pageInfo: PageInfo;
  nodes: Post[];
}

export interface AddPostInput {
  title: string;
  content: string;
  isPublished?: boolean;
  description?: string;
  slug?: string;
}

export interface UpdatePostInput {
  id: UUID;
  title: string;
  content: string;
  isPublished?: boolean;
  description?: string;
  slug?: string;
}

export interface DeletePostInput {
  id: UUID;
}

export type PostMutationResult = Post | DeletedPost | DbError | AuthError;

export type PostSortBy = "CREATED_AT" | "UPDATED_AT" | "TITLE";
export type SortDirection = "ASC" | "DESC";
export interface PostSortParams {
  sortBy: PostSortBy;
  sortDirection: SortDirection;
}
