import { Handlers } from "$fresh/server.ts";
import { logger } from "@/utils/logger.ts";

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

function getClientIp(req: Request): string | null {
  return req.headers.get("x-forwarded-for")?.split(",")[0]?.trim() ?? null;
}

const AUTH_ERROR_STRINGS = [
  "Token not found",
  "Token expired",
  "Invalid token",
];

function isAuthMessage(msg: unknown): boolean {
  return typeof msg === "string" &&
    AUTH_ERROR_STRINGS.some((s) => msg.includes(s));
}

// Check if response is auth error (top-level errors or mutation AuthError payloads)
function isAuthError(json: unknown): boolean {
  if (!json || typeof json !== "object") return false;
  const root = json as Record<string, unknown>;

  // Top-level GraphQL errors
  if (Array.isArray(root.errors)) {
    if (
      (root.errors as Array<{ message?: unknown }>).some((e) =>
        isAuthMessage(e.message)
      )
    ) {
      return true;
    }
  }

  // Mutation/query payload AuthError union variants: data.*.message
  if (root.data && typeof root.data === "object") {
    for (const value of Object.values(root.data as Record<string, unknown>)) {
      if (value && typeof value === "object") {
        if (isAuthMessage((value as Record<string, unknown>).message)) {
          return true;
        }
      }
    }
  }

  return false;
}

export const handler: Handlers = {
  async POST(req) {
    const endpoint = Deno.env.get("GRAPHQL_ENDPOINT") ||
      "http://localhost:8000/graphql";

    const requestId = crypto.randomUUID();
    const ip = getClientIp(req);
    const path = new URL(req.url).pathname;

    const cookieHeader = req.headers.get("Cookie");
    const headers: HeadersInit = {
      "Content-Type": "application/json",
      "X-Request-ID": requestId,
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
        logger.warn("auth.token_invalid", {
          request_id: requestId,
          who: { ip },
          where: { path, method: "POST" },
        });

        const refreshToken = getRefreshToken(cookieHeader);
        if (refreshToken) {
          logger.info("auth.refresh_attempt", {
            request_id: requestId,
            who: { ip },
          });

          // Attempt refresh
          const refreshResponse = await fetch(endpoint, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              "Cookie": cookieHeader || "",
              "X-Request-ID": requestId,
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
              logger.info("auth.refresh_success", {
                request_id: requestId,
                who: { ip },
                what: { outcome: "success" },
              });

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
                  "X-Request-ID": requestId,
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

          logger.warn("auth.refresh_failed", {
            request_id: requestId,
            who: { ip },
            what: { outcome: "failure" },
          });

          logger.warn("auth.session_expired", {
            request_id: requestId,
            who: { ip },
            where: { path, method: "POST", status: 401 },
          });

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
