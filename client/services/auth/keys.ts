export const authKeys = {
  all: ["auth"] as const,
  user: () => [...authKeys.all, "user"] as const,
  apiKeys: () => [...authKeys.all, "apiKeys"] as const,
};
