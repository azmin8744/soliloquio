import { useEffect, useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useMe, useUpdateUser } from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { Input } from "../components/Input.tsx";
import { getQueryClient } from "../utils/query_client.ts";

function UserSettingsFormContent() {
  const { data: user, isLoading } = useMe();
  const [email, setEmail] = useState("");
  const [message, setMessage] = useState<
    { text: string; type: "success" | "error" } | null
  >(null);

  useEffect(() => {
    if (user?.email) setEmail(user.email);
  }, [user?.email]);

  const updateUserMutation = useUpdateUser();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setMessage(null);
    updateUserMutation.mutate(
      { email },
      {
        onSuccess: (data) => {
          if ("email" in data) {
            setMessage({ text: "Email updated successfully", type: "success" });
          } else if ("message" in data) {
            setMessage({
              text: (data as { message: string }).message,
              type: "error",
            });
          }
        },
        onError: (error) => {
          setMessage({
            text: (error as Error).message || "An unexpected error occurred",
            type: "error",
          });
        },
      },
    );
  };

  if (isLoading) {
    return <div class="text-gray-500 text-sm">Loading...</div>;
  }

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
      <Input
        id="email"
        label="Email"
        type="email"
        value={email}
        onInput={(e) => setEmail(e.currentTarget.value)}
        required
      />

      {message && (
        <div
          class={`rounded-md p-4 ${
            message.type === "success" ? "bg-green-50" : "bg-red-50"
          }`}
        >
          <div class="flex">
            <div class="ml-3">
              <h3
                class={`text-sm font-medium ${
                  message.type === "success" ? "text-green-800" : "text-red-800"
                }`}
              >
                {message.type === "success" ? "Success" : "Error"}
              </h3>
              <div
                class={`mt-2 text-sm ${
                  message.type === "success" ? "text-green-700" : "text-red-700"
                }`}
              >
                <p>{message.text}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      <div>
        <Button
          type="submit"
          className="w-full flex justify-center"
          isLoading={updateUserMutation.isPending}
        >
          Save Changes
        </Button>
      </div>
    </form>
  );
}

export default function UserSettingsForm() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <UserSettingsFormContent />
    </QueryClientProvider>
  );
}
