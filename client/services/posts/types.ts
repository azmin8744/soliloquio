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
}

export interface UpdatePostInput {
  id: UUID;
  title: string;
  content: string;
  isPublished?: boolean;
}

export interface DeletePostInput {
  id: UUID;
}

export type PostMutationResult = Post | DeletedPost | DbError | AuthError;
