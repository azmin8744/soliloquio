import { Handlers } from "$fresh/server.ts";

const LOGOUT_MUTATION = `
  mutation Logout($refreshToken: String!) {
    logout(refreshToken: $refreshToken)
  }
`;

// Extract refresh token from cookies
function getRefreshToken(cookieHeader: string | null): string | null {
  if (!cookieHeader) return null;
  const match = cookieHeader.match(/refresh_token=([^;]+)/);
  return match ? match[1] : null;
}

export const handler: Handlers = {
  async POST(req) {
    const endpoint = Deno.env.get("GRAPHQL_ENDPOINT") ||
      "http://localhost:8000/graphql";

    const cookieHeader = req.headers.get("Cookie");
    const refreshToken = getRefreshToken(cookieHeader);

    // Call backend logout if we have a refresh token
    if (refreshToken) {
      await fetch(endpoint, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Cookie": cookieHeader || "",
        },
        body: JSON.stringify({
          query: LOGOUT_MUTATION,
          variables: { refreshToken },
        }),
      });
    }

    // Clear cookies by setting expired ones
    const headers = new Headers({ "Content-Type": "application/json" });
    headers.append(
      "Set-Cookie",
      "access_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; SameSite=Lax",
    );
    headers.append(
      "Set-Cookie",
      "refresh_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; SameSite=Lax",
    );

    return new Response(JSON.stringify({ success: true }), {
      status: 200,
      headers,
    });
  },
};
