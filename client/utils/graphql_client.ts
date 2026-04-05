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
      throw new Error(
        response.status === 401
          ? "Session expired"
          : `GraphQL request failed: ${response.statusText}`,
      );
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
