import { Handlers } from "$fresh/server.ts";

const REFRESH_MUTATION = `
  mutation RefreshAccessToken($refreshToken: String!) {
    refreshAccessToken(refreshToken: $refreshToken) {
      ... on AuthorizedUser { token refreshToken }
      ... on AuthError { message }
    }
  }
`;

// Extract refresh token from cookies
function getRefreshToken(cookieHeader: string | null): string | null {
  if (!cookieHeader) return null;
  const match = cookieHeader.match(/refresh_token=([^;]+)/);
  return match ? match[1] : null;
}

// Check if response is auth error
function isAuthError(json: unknown): boolean {
  if (!json || typeof json !== "object") return false;
  const data = json as Record<string, unknown>;
  if (!data.errors) return false;
  const errors = data.errors as Array<{ message?: string }>;
  return errors.some((e) =>
    e.message?.includes("Token not found") ||
    e.message?.includes("Token expired") ||
    e.message?.includes("Invalid token")
  );
}

export const handler: Handlers = {
  async POST(req) {
    const endpoint = Deno.env.get("GRAPHQL_ENDPOINT") ||
      "http://localhost:8000/graphql";

    const cookieHeader = req.headers.get("Cookie");
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    };

    // Forward cookies to backend
    if (cookieHeader) {
      headers["Cookie"] = cookieHeader;
    }

    // Keep Authorization header for backward compat during migration
    const authHeader = req.headers.get("Authorization");
    if (authHeader) {
      headers["Authorization"] = authHeader;
    }

    const body = await req.text();

    let response = await fetch(endpoint, {
      method: "POST",
      headers,
      body,
    });

    // Check for auth error - try refresh
    if (response.ok) {
      const clonedResponse = response.clone();
      const json = await clonedResponse.json();

      if (isAuthError(json)) {
        const refreshToken = getRefreshToken(cookieHeader);
        if (refreshToken) {
          // Attempt refresh
          const refreshResponse = await fetch(endpoint, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              "Cookie": cookieHeader || "",
            },
            body: JSON.stringify({
              query: REFRESH_MUTATION,
              variables: { refreshToken },
            }),
          });

          if (refreshResponse.ok) {
            const refreshJson = await refreshResponse.json();
            const refreshData = refreshJson?.data?.refreshAccessToken;

            if (refreshData?.token) {
              // Refresh succeeded - collect Set-Cookie headers, retry original request
              const setCookieHeaders: string[] = [];
              refreshResponse.headers.forEach((value, key) => {
                if (key.toLowerCase() === "set-cookie") {
                  setCookieHeaders.push(value);
                }
              });

              // Retry with new cookies
              const newCookieHeader = `access_token=${refreshData.token}; ${
                cookieHeader || ""
              }`;
              response = await fetch(endpoint, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                  "Cookie": newCookieHeader,
                },
                body,
              });

              // Forward Set-Cookie from refresh to client
              const responseHeaders = new Headers({
                "Content-Type": "application/json",
              });
              setCookieHeaders.forEach((cookie) => {
                responseHeaders.append("Set-Cookie", cookie);
              });

              return new Response(response.body, {
                status: response.status,
                headers: responseHeaders,
              });
            }
          }

          // Refresh failed - return 401 to client
          return new Response(
            JSON.stringify({ errors: [{ message: "Session expired" }] }),
            {
              status: 401,
              headers: { "Content-Type": "application/json" },
            },
          );
        }
      }
    }

    // Forward Set-Cookie headers from backend to browser
    const responseHeaders = new Headers({ "Content-Type": "application/json" });
    response.headers.forEach((value, key) => {
      if (key.toLowerCase() === "set-cookie") {
        responseHeaders.append("Set-Cookie", value);
      }
    });

    return new Response(response.body, {
      status: response.status,
      headers: responseHeaders,
    });
  },
};
