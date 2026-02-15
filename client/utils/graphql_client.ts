const ENDPOINT = "/api/graphql";

function getEndpoint(): string {
  return ENDPOINT;
}

// Simple GraphQL client that works in both browser and server
class SimpleGraphQLClient {
  private endpoint: string;

  constructor(endpoint: string) {
    this.endpoint = endpoint;
  }

  async request<T>(
    query: string,
    variables?: Record<string, unknown>,
  ): Promise<T> {
    const response = await fetch(this.endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      credentials: "include", // Send cookies with requests
      body: JSON.stringify({ query, variables }),
    });

    if (!response.ok) {
      if (response.status === 401) {
        // Save editor buffer to localStorage before redirect
        if (typeof globalThis !== "undefined" && globalThis.location) {
          try {
            const { activePostId, editorBuffer, isDirty } = await import(
              "./workspace_signals.ts"
            );
            if (isDirty.value && activePostId.value) {
              localStorage.setItem(
                "soliloquio_editor_recovery",
                JSON.stringify({
                  postId: activePostId.value,
                  buffer: editorBuffer.value,
                }),
              );
            }
          } catch { /* signals may not be loaded */ }
          globalThis.location.href = "/auth/signin";
        }
        throw new Error("Session expired");
      }
      throw new Error(`GraphQL request failed: ${response.statusText}`);
    }

    const json = await response.json();

    if (json.errors && json.errors.length > 0) {
      throw new Error(json.errors[0].message);
    }

    return json.data;
  }
}

export function getGraphQLClient() {
  return new SimpleGraphQLClient(getEndpoint());
}
