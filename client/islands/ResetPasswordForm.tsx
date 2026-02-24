import { useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useResetPassword } from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { PasswordInput } from "../components/PasswordInput.tsx";
import { getQueryClient } from "../utils/query_client.ts";

interface Props {
  token: string;
}

function ResetPasswordFormContent({ token }: Props) {
  const [newPassword, setNewPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [message, setMessage] = useState<
    { text: string; type: "success" | "error" } | null
  >(null);

  const mutation = useResetPassword();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setMessage(null);
    if (newPassword !== confirm) {
      setMessage({ text: "Passwords do not match", type: "error" });
      return;
    }
    mutation.mutate(
      { token, newPassword },
      {
        onSuccess: (data) => {
          if ("message" in data) {
            setMessage({
              text: (data as { message: string }).message,
              type: "success",
            });
            setTimeout(() => {
              globalThis.location.href = "/auth/signin";
            }, 2000);
          }
        },
        onError: (err) => {
          setMessage({
            text: (err as Error).message || "An unexpected error occurred",
            type: "error",
          });
        },
      },
    );
  };

  if (!token) {
    return <p class="text-sm text-red-700">Invalid or missing reset token.</p>;
  }

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
      <PasswordInput
        id="new-password"
        label="New password"
        value={newPassword}
        onInput={(e) => setNewPassword(e.currentTarget.value)}
        required
      />
      <PasswordInput
        id="confirm-password"
        label="Confirm new password"
        value={confirm}
        onInput={(e) => setConfirm(e.currentTarget.value)}
        required
      />

      {message && (
        <div
          class={`rounded-md p-4 ${
            message.type === "success" ? "bg-green-50" : "bg-red-50"
          }`}
        >
          <p
            class={`text-sm ${
              message.type === "success" ? "text-green-800" : "text-red-700"
            }`}
          >
            {message.text}
          </p>
        </div>
      )}

      <Button
        type="submit"
        className="w-full flex justify-center"
        isLoading={mutation.isPending}
      >
        Reset password
      </Button>
    </form>
  );
}

export default function ResetPasswordForm({ token }: Props) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <ResetPasswordFormContent token={token} />
    </QueryClientProvider>
  );
}
