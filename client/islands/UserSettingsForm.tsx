import { QueryClientProvider } from "@tanstack/react-query";
import { getQueryClient } from "../utils/query_client.ts";
import UserSettingsFormContent from "./UserSettingsFormContent.tsx";
import ApiKeysSection from "./ApiKeysSection.tsx";

export default function UserSettingsForm() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <div class="space-y-10">
        <UserSettingsFormContent />
        <hr class="border-gray-200" />
        <ApiKeysSection />
      </div>
    </QueryClientProvider>
  );
}
