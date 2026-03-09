import { assertEquals } from "$std/assert/mod.ts";
import { isTraversalPath } from "./path_guard.ts";

Deno.test("rejects .. segment", () => {
  assertEquals(isTraversalPath("../etc/passwd"), true);
});

Deno.test("rejects .. in middle", () => {
  assertEquals(isTraversalPath("uuid/../etc/passwd"), true);
});

Deno.test("rejects .. at end", () => {
  assertEquals(isTraversalPath("uuid/.."), true);
});

Deno.test("rejects . segment", () => {
  assertEquals(isTraversalPath("./etc/passwd"), true);
});

Deno.test("rejects . in middle", () => {
  assertEquals(isTraversalPath("uuid/./file.webp"), true);
});

Deno.test("allows normal uuid/file path", () => {
  assertEquals(isTraversalPath("550e8400-e29b-41d4-a716-446655440000/thumbnail.webp"), false);
});

Deno.test("allows single segment", () => {
  assertEquals(isTraversalPath("thumbnail.webp"), false);
});
