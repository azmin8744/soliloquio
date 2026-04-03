export interface PublicAuthor {
  id: string;
  display_name: string | null;
  bio: string | null;
}

export interface PrevNextPost {
  id: string;
  title: string;
  slug: string | null;
  first_published_at: string | null;
}

export interface PublicPost {
  id: string;
  title: string;
  description: string | null;
  slug: string | null;
  cover_image: string | null;
  content: string;
  first_published_at: string | null;
  created_at: string;
  updated_at: string;
  author: PublicAuthor;
  prev_post: PrevNextPost | null;
  next_post: PrevNextPost | null;
}

export interface PageInfo {
  hasNextPage: boolean;
  hasPreviousPage: boolean;
}

export interface PostConnection {
  pageInfo: PageInfo;
  nodes: PublicPost[];
  pageNumber: number | null;
  totalPages: number | null;
}
