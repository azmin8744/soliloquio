import { Handlers } from "$fresh/server.ts";

export const handler: Handlers = {
  async POST(req) {
    const backend = Deno.env.get("BACKEND_BASE") ||
      "http://localhost:8000";
    const cookieHeader = req.headers.get("Cookie");
    const headers: HeadersInit = {};
    if (cookieHeader) headers["Cookie"] = cookieHeader;
    const contentType = req.headers.get("Content-Type");
    if (contentType) headers["Content-Type"] = contentType;

    const res = await fetch(`${backend}/upload`, {
      method: "POST",
      headers,
      body: req.body,
      // @ts-ignore - duplex needed for streaming
      duplex: "half",
    });

    const responseHeaders = new Headers({
      "Content-Type": "application/json",
    });
    res.headers.forEach((v, k) => {
      if (k.toLowerCase() === "set-cookie") {
        responseHeaders.append("Set-Cookie", v);
      }
    });

    return new Response(res.body, {
      status: res.status,
      headers: responseHeaders,
    });
  },
};
