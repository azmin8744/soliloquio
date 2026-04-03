import { getPublicClient, resolveAssetUrl } from "../../utils/graphql_client.ts";
import { PostConnection, PublicAuthor, PublicPost } from "./types.ts";

const GET_POSTS_QUERY = `
  query GetPosts($page: Int, $first: Int) {
    posts(page: $page, first: $first) {
      pageInfo {
        hasNextPage
        hasPreviousPage
      }
      pageNumber
      totalPages
      nodes {
        id
        title
        description
        slug
        coverImage
        content
        firstPublishedAt
        author {
          displayName
        }
      }
    }
  }
`;

const GET_POST_QUERY = `
  query GetPost($id: UUID, $slug: String) {
    post(id: $id, slug: $slug) {
      id
      title
      description
      slug
      coverImage
      content
      firstPublishedAt
      createdAt
      updatedAt
      author {
        id
        displayName
        bio
      }
      prevPost {
        id
        title
        slug
        firstPublishedAt
      }
      nextPost {
        id
        title
        slug
        firstPublishedAt
      }
    }
  }
`;

const GET_AUTHOR_QUERY = `
  query GetAuthor {
    author {
      id
      displayName
      bio
    }
  }
`;

// GraphQL returns camelCase; map to our snake_case types
// deno-lint-ignore no-explicit-any
function mapPost(p: any): PublicPost {
  return {
    id: p.id,
    title: p.title,
    description: p.description ?? null,
    slug: p.slug ?? null,
    cover_image: resolveAssetUrl(p.coverImage ?? null),
    content: p.content ?? "",
    first_published_at: p.firstPublishedAt ?? null,
    created_at: p.createdAt ?? "",
    updated_at: p.updatedAt ?? "",
    author: {
      id: p.author?.id ?? "",
      display_name: p.author?.displayName ?? null,
      bio: p.author?.bio ?? null,
    },
    prev_post: p.prevPost
      ? {
        id: p.prevPost.id,
        title: p.prevPost.title,
        slug: p.prevPost.slug ?? null,
        first_published_at: p.prevPost.firstPublishedAt ?? null,
      }
      : null,
    next_post: p.nextPost
      ? {
        id: p.nextPost.id,
        title: p.nextPost.title,
        slug: p.nextPost.slug ?? null,
        first_published_at: p.nextPost.firstPublishedAt ?? null,
      }
      : null,
  };
}

export async function getPosts(
  { page, first }: { page?: number; first?: number } = {},
): Promise<PostConnection> {
  const client = getPublicClient();
  // deno-lint-ignore no-explicit-any
  const data = await client.request<{ posts: any }>(GET_POSTS_QUERY, {
    page,
    first,
  });
  return {
    pageInfo: data.posts.pageInfo,
    nodes: data.posts.nodes.map(mapPost),
    pageNumber: data.posts.pageNumber ?? null,
    totalPages: data.posts.totalPages ?? null,
  };
}

const UUID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export async function getPost(slugOrId: string): Promise<PublicPost | null> {
  const client = getPublicClient();
  const isId = UUID_RE.test(slugOrId);
  // deno-lint-ignore no-explicit-any
  const data = await client.request<{ post: any | null }>(GET_POST_QUERY, {
    id: isId ? slugOrId : undefined,
    slug: isId ? undefined : slugOrId,
  });
  return data.post ? mapPost(data.post) : null;
}

export async function getAuthor(): Promise<PublicAuthor> {
  const client = getPublicClient();
  // deno-lint-ignore no-explicit-any
  const data = await client.request<{ author: any }>(GET_AUTHOR_QUERY);
  return {
    id: data.author.id,
    display_name: data.author.displayName ?? null,
    bio: data.author.bio ?? null,
  };
}
