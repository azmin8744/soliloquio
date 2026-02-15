import { QueryClientProvider } from "@tanstack/react-query";
import { ComponentChildren } from "preact";
import { getQueryClient } from "../utils/query_client.ts";

export default function QueryProvider(
  { children }: { children: ComponentChildren },
) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      {children}
    </QueryClientProvider>
  );
}
