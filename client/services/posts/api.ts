// GraphQL queries are just strings, no special library needed
import { getGraphQLClient } from "../../utils/graphql_client.ts";
import {
  AddPostInput,
  DeletePostInput,
  PostMutationResult,
  UpdatePostInput,
} from "./types.ts";
import { Post } from "../../domains/posts.ts";
import { UUID } from "../../domains/common.ts";

const ADD_POST_MUTATION = `
  mutation AddPost($newPost: AddPostInput!) {
    addPost(newPost: $newPost) {
      ... on Post {
        id
        title
        content
        markdownContent
        isPublished
        createdAt
        updatedAt
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const UPDATE_POST_MUTATION = `
  mutation UpdatePost($post: UpdatePostInput!) {
    updatePost(post: $post) {
      ... on Post {
        id
        title
        content
        markdownContent
        isPublished
        createdAt
        updatedAt
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const DELETE_POST_MUTATION = `
  mutation DeletePost($post: DeletePostInput!) {
    deletePost(post: $post) {
      ... on DeletedPost {
        id
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const GET_POSTS_QUERY = `
  query GetPosts {
    posts {
      id
      title
      isPublished
      createdAt
      updatedAt
      markdownContent
      content
    }
  }
`;

const GET_POST_QUERY = `
  query GetPost($id: UUID!) {
    post(id: $id) {
      id
      title
      content
      markdownContent
      isPublished
      createdAt
      updatedAt
    }
  }
`;

export async function addPost(
  input: AddPostInput,
): Promise<PostMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ addPost: PostMutationResult }>(
    ADD_POST_MUTATION,
    { newPost: input },
  );
  return data.addPost;
}

export async function updatePost(
  input: UpdatePostInput,
): Promise<PostMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ updatePost: PostMutationResult }>(
    UPDATE_POST_MUTATION,
    { post: input },
  );
  return data.updatePost;
}

export async function deletePost(
  input: DeletePostInput,
): Promise<PostMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ deletePost: PostMutationResult }>(
    DELETE_POST_MUTATION,
    { post: input },
  );
  return data.deletePost;
}

export async function getPosts(): Promise<Post[]> {
  const client = getGraphQLClient();
  const data = await client.request<{ posts: Post[] }>(GET_POSTS_QUERY);
  return data.posts;
}

export async function getPost(id: UUID): Promise<Post | null> {
  const client = getGraphQLClient();
  const data = await client.request<{ post: Post | null }>(GET_POST_QUERY, {
    id,
  });
  return data.post;
}
