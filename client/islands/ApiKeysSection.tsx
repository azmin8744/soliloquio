import { useState } from "preact/hooks";
import {
  useApiKeys,
  useCreateApiKey,
  useRevokeApiKey,
} from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { CreateApiKeyResult } from "../domains/users.ts";

export default function ApiKeysSection() {
  const { data: keys, isLoading } = useApiKeys();
  const createMutation = useCreateApiKey();
  const revokeMutation = useRevokeApiKey();
  const [label, setLabel] = useState("");
  const [newKey, setNewKey] = useState<CreateApiKeyResult | null>(null);
  const [copied, setCopied] = useState(false);

  const handleCreate = (e: Event) => {
    e.preventDefault();
    if (!label.trim()) return;
    createMutation.mutate(label.trim(), {
      onSuccess: (data) => {
        if ("rawKey" in data) {
          setNewKey(data as CreateApiKeyResult);
          setLabel("");
        }
      },
    });
  };

  const handleCopy = () => {
    if (newKey) {
      navigator.clipboard.writeText(newKey.rawKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div class="space-y-4">
      <h3 class="text-lg font-medium text-gray-900">API Keys</h3>
      <p class="text-sm text-gray-500">
        API keys grant read-only access to your published posts via the public
        endpoint.
      </p>

      {newKey && (
        <div class="rounded-md bg-green-50 p-4">
          <p class="text-sm font-medium text-green-800 mb-2">
            API key created — copy it now, it won't be shown again.
          </p>
          <div class="flex items-center gap-2">
            <code class="flex-1 text-xs bg-white border rounded px-2 py-1 overflow-x-auto">
              {newKey.rawKey}
            </code>
            <button
              type="button"
              class="text-sm text-green-700 underline"
              onClick={handleCopy}
            >
              {copied ? "Copied!" : "Copy"}
            </button>
          </div>
          <button
            type="button"
            class="mt-2 text-xs text-gray-500 underline"
            onClick={() => setNewKey(null)}
          >
            Dismiss
          </button>
        </div>
      )}

      <form onSubmit={handleCreate} class="flex gap-2">
        <input
          class="flex-1 rounded-md border-gray-300 border px-3 py-2 text-sm shadow-sm"
          placeholder="Key label (e.g. My Frontend)"
          value={label}
          onInput={(e) => setLabel(e.currentTarget.value)}
          required
        />
        <Button type="submit" isLoading={createMutation.isPending}>
          Create
        </Button>
      </form>

      {isLoading
        ? <p class="text-sm text-gray-500">Loading keys...</p>
        : (
          <ul class="divide-y divide-gray-200 border rounded-md">
            {keys && keys.length === 0 && (
              <li class="px-4 py-3 text-sm text-gray-400">No API keys yet.</li>
            )}
            {keys?.map((key) => (
              <li
                key={key.id}
                class="flex items-center justify-between px-4 py-3"
              >
                <div>
                  <p class="text-sm font-medium text-gray-900">{key.label}</p>
                  <p class="text-xs text-gray-500">
                    Created {new Date(key.createdAt).toLocaleDateString()}
                    {key.lastUsedAt &&
                      ` · Last used ${
                        new Date(key.lastUsedAt).toLocaleDateString()
                      }`}
                  </p>
                </div>
                <button
                  type="button"
                  class="text-sm text-red-600 hover:text-red-800 disabled:opacity-50"
                  onClick={() => revokeMutation.mutate(key.id)}
                  disabled={revokeMutation.isPending}
                >
                  Revoke
                </button>
              </li>
            ))}
          </ul>
        )}
    </div>
  );
}
