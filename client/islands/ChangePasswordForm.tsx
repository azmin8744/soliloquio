import { useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useChangePassword } from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { PasswordInput } from "../components/PasswordInput.tsx";
import { getQueryClient } from "../utils/query_client.ts";

function ChangePasswordFormContent() {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmNewPassword, setConfirmNewPassword] = useState("");
  const [message, setMessage] = useState<
    { text: string; type: "success" | "error" } | null
  >(null);

  const changePasswordMutation = useChangePassword();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setMessage(null);

    if (newPassword !== confirmNewPassword) {
      setMessage({ text: "New passwords do not match", type: "error" });
      return;
    }

    changePasswordMutation.mutate(
      { currentPassword, newPassword },
      {
        onSuccess: (data) => {
          if ("message" in data) {
            setMessage({
              text: (data as { message: string }).message,
              type: "success",
            }); // Optimistic
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

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
      <PasswordInput
        id="currentDetails"
        label="Current Password"
        value={currentPassword}
        onInput={(e) => setCurrentPassword(e.currentTarget.value)}
        required
      />

      <PasswordInput
        id="newPassword"
        label="New Password"
        value={newPassword}
        onInput={(e) => setNewPassword(e.currentTarget.value)}
        required
      />

      <PasswordInput
        id="confirmNewPassword"
        label="Confirm New Password"
        value={confirmNewPassword}
        onInput={(e) => setConfirmNewPassword(e.currentTarget.value)}
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
          isLoading={changePasswordMutation.isPending}
        >
          Change Password
        </Button>
      </div>
    </form>
  );
}

export default function ChangePasswordForm() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <ChangePasswordFormContent />
    </QueryClientProvider>
  );
}
