import { NaiveDateTime, UUID } from "./common.ts";

export interface Post {
  id: UUID;
  title: string;
  isPublished: boolean;
  firstPublishedAt?: NaiveDateTime;
  createdAt?: NaiveDateTime;
  updatedAt?: NaiveDateTime;
  markdownContent: string;
  content: string;
  description?: string;
  slug?: string;
}

export interface DeletedPost {
  id: UUID;
}
