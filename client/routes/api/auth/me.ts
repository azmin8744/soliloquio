import { Handlers } from "$fresh/server.ts";

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

    const cookieHeader = req.headers.get("Cookie");
    if (!cookieHeader) {
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Cookie": cookieHeader,
      },
      body: JSON.stringify({ query: ME_QUERY }),
    });

    if (!response.ok) {
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    const json = await response.json();

    if (json.errors || !json.data?.me) {
      return new Response(JSON.stringify({ authenticated: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }

    return new Response(
      JSON.stringify({ authenticated: true, user: json.data.me }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  },
};
