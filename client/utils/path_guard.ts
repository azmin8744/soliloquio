/** Returns true if the path contains traversal segments (.. or .) */
export function isTraversalPath(path: string): boolean {
  return path.split("/").some((seg) => seg === ".." || seg === ".");
}
