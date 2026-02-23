import type { PostSortParams } from "./types.ts";

export const postKeys = {
  all: ["posts"] as const,
  lists: (sort?: PostSortParams, search?: string) =>
    sort !== undefined || search !== undefined
      ? [...postKeys.all, "list", sort ?? null, search ?? null] as const
      : [...postKeys.all, "list"] as const,
  list: (filters: string) => [...postKeys.lists(), { filters }] as const,
  details: () => [...postKeys.all, "detail"] as const,
  detail: (id: string) => [...postKeys.details(), id] as const,
};
