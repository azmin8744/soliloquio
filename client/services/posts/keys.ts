import type { PostSortParams } from "./types.ts";

export const postKeys = {
  all: ["posts"] as const,
  lists: (sort?: PostSortParams) =>
    sort ? [...postKeys.all, "list", sort] as const : [...postKeys.all, "list"] as const,
  list: (filters: string) => [...postKeys.lists(), { filters }] as const,
  details: () => [...postKeys.all, "detail"] as const,
  detail: (id: string) => [...postKeys.details(), id] as const,
};
