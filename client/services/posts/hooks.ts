import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { addPost, deletePost, getPost, getPosts, updatePost } from "./api.ts";
import { AddPostInput, DeletePostInput, UpdatePostInput } from "./types.ts";
import { postKeys } from "./keys.ts";
import { UUID } from "../../domains/common.ts";

export function usePosts() {
  return useQuery({
    queryKey: postKeys.lists(),
    queryFn: async () => {
      return await getPosts();
    },
  });
}

export function usePost(id: UUID) {
  return useQuery({
    queryKey: postKeys.detail(id),
    queryFn: async () => {
      return await getPost(id);
    },
    enabled: !!id,
  });
}

export function useCreatePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: AddPostInput) => addPost(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: postKeys.lists() });
    },
  });
}

export function useUpdatePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdatePostInput) => updatePost(input),
    onSuccess: (data) => {
      if ("id" in data) {
        queryClient.invalidateQueries({ queryKey: postKeys.detail(data.id) });
        queryClient.invalidateQueries({ queryKey: postKeys.lists() });
      }
    },
  });
}

export function useDeletePost() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: DeletePostInput) => deletePost(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: postKeys.lists() });
    },
  });
}
