import type { Handlers, RouteConfig } from "$fresh/server.ts";

export const config: RouteConfig = { skipInheritedLayouts: true };

export const handler: Handlers = {
  GET: (req) => Response.redirect(new URL("/posts", req.url), 302),
};
