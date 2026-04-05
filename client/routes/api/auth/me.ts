import { Handlers } from "$fresh/server.ts";
import { logger } from "@/utils/logger.ts";

const ME_QUERY = `
  query Me {
    me {
      id
      email
      createdAt
      updatedAt
    }
  }
`;

export const handler: Handlers = {
  async GET(req) {
    const endpoint = Deno.env.get("GRAPHQL_ENDPOINT") ||
      "http://localhost:8000/graphql";

    const ip = req.headers.get("x-forwarded-for")?.split(",")[0]?.trim() ??
      null;
    const cookieHeader = req.headers.get("Cookie");
    if (!cookieHeader) {
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    const meResponse = await fetch(endpoint, {
      method: "POST",
      headers: { "Content-Type": "application/json", "Cookie": cookieHeader },
      body: JSON.stringify({ query: ME_QUERY }),
    });

    if (!meResponse.ok) {
      logger.warn("auth.me_unauthenticated", {
        who: { ip },
        what: { reason: "backend_error" },
      });
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    const json = await meResponse.json();
    if (json.errors || !json.data?.me) {
      logger.warn("auth.me_unauthenticated", {
        who: { ip },
        what: { reason: json.errors?.[0]?.message ?? "no_user" },
      });
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    return new Response(
      JSON.stringify({ authenticated: true, user: json.data.me }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  },
};
