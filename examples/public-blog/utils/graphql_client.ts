class PublicGraphQLClient {
  private endpoint: string;
  private apiKey: string;

  constructor(endpoint: string, apiKey: string) {
    this.endpoint = endpoint;
    this.apiKey = apiKey;
  }

  async request<T>(
    query: string,
    variables?: Record<string, unknown>,
  ): Promise<T> {
    const response = await fetch(this.endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": this.apiKey,
      },
      body: JSON.stringify({ query, variables }),
    });

    if (!response.ok) {
      throw new Error(`GraphQL request failed: ${response.statusText}`);
    }

    const json = await response.json();

    if (json.errors && json.errors.length > 0) {
      throw new Error(json.errors[0].message);
    }

    return json.data;
  }
}

let client: PublicGraphQLClient | null = null;

export function getPublicClient(): PublicGraphQLClient {
  if (!client) {
    const endpoint = Deno.env.get("PUBLIC_API_URL") ??
      "http://localhost:8000/public";
    const apiKey = Deno.env.get("PUBLIC_API_KEY") ?? "";
    client = new PublicGraphQLClient(endpoint, apiKey);
  }
  return client;
}

export function resolveAssetUrl(coverImage: string | null): string | null {
  if (!coverImage) return null;
  if (coverImage.startsWith("http")) return coverImage;
  const base = Deno.env.get("PUBLIC_API_URL") ?? "http://localhost:8000/public";
  const origin = new URL(base).origin;
  return `${origin}${coverImage}`;
}
