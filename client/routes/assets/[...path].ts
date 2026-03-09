import { Handlers } from "$fresh/server.ts";
import { isTraversalPath } from "@/utils/path_guard.ts";

export const handler: Handlers = {
  async GET(req, ctx) {
    const backend = Deno.env.get("BACKEND_BASE") || "http://localhost:8000";
    const path = ctx.params.path;
    if (isTraversalPath(path)) {
      return new Response(null, { status: 400 });
    }
    const res = await fetch(`${backend}/assets/${path}`);
    if (!res.ok) {
      return new Response(null, { status: res.status });
    }
    return new Response(res.body, {
      status: 200,
      headers: { "Content-Type": "image/webp" },
    });
  },
};
