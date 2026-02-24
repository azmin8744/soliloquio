import { useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useForgotPassword } from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { Input } from "../components/Input.tsx";
import { getQueryClient } from "../utils/query_client.ts";

function ForgotPasswordFormContent() {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  const [error, setError] = useState("");

  const mutation = useForgotPassword();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setError("");
    mutation.mutate(email, {
      onSuccess: (data) => {
        if ("message" in data) setSent(true);
        else setError("An unexpected error occurred");
      },
      onError: (err) => {
        setError((err as Error).message || "An unexpected error occurred");
      },
    });
  };

  if (sent) {
    return (
      <div class="rounded-md bg-green-50 p-4">
        <p class="text-sm text-green-800">
          If that email exists, a reset link has been sent. Check your inbox.
        </p>
        <p class="mt-2 text-sm">
          <a href="/auth/signin" class="text-blue-600 hover:text-blue-500">
            Back to sign in
          </a>
        </p>
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
      <Input
        id="email"
        type="email"
        label="Email address"
        value={email}
        onInput={(e) => setEmail(e.currentTarget.value)}
        required
      />

      {error && (
        <div class="rounded-md bg-red-50 p-4">
          <p class="text-sm text-red-700">{error}</p>
        </div>
      )}

      <Button
        type="submit"
        className="w-full flex justify-center"
        isLoading={mutation.isPending}
      >
        Send reset link
      </Button>

      <p class="text-center text-sm">
        <a href="/auth/signin" class="text-blue-600 hover:text-blue-500">
          Back to sign in
        </a>
      </p>
    </form>
  );
}

export default function ForgotPasswordForm() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <ForgotPasswordFormContent />
    </QueryClientProvider>
  );
}
