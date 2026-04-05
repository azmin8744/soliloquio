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

export const handler: Handlers = {
  async POST(req) {
    const endpoint = Deno.env.get("GRAPHQL_ENDPOINT") ||
      "http://localhost:8000/graphql";

    const ip = req.headers.get("x-forwarded-for")?.split(",")[0]?.trim() ??
      null;
    const cookieHeader = req.headers.get("Cookie");
    const refreshToken = getRefreshToken(cookieHeader);

    if (!refreshToken) {
      return new Response(
        JSON.stringify({ success: false, error: "No refresh token" }),
        {
          status: 401,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    logger.info("auth.refresh_attempt", { who: { ip } });

    const response = await fetch(endpoint, {
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

    if (!response.ok) {
      logger.warn("auth.refresh_failed", {
        who: { ip },
        what: { outcome: "failure", reason: "backend_error" },
      });
      return new Response(
        JSON.stringify({ success: false, error: "Refresh failed" }),
        {
          status: 401,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    // Forward Set-Cookie headers from backend
    const responseHeaders = new Headers({ "Content-Type": "application/json" });
    response.headers.forEach((value, key) => {
      if (key.toLowerCase() === "set-cookie") {
        responseHeaders.append("Set-Cookie", value);
      }
    });

    const json = await response.json();
    const refreshData = json?.data?.refreshAccessToken;

    if (refreshData?.token) {
      logger.info("auth.refresh_success", {
        who: { ip },
        what: { outcome: "success" },
      });
      return new Response(JSON.stringify({ success: true }), {
        status: 200,
        headers: responseHeaders,
      });
    }

    logger.warn("auth.refresh_failed", {
      who: { ip },
      what: { outcome: "failure", reason: refreshData?.message ?? "unknown" },
    });

    return new Response(
      JSON.stringify({
        success: false,
        error: refreshData?.message || "Refresh failed",
      }),
      {
        status: 401,
        headers: { "Content-Type": "application/json" },
      },
    );
  },
};
