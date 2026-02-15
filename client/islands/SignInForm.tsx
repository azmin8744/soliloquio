import { useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useSignIn } from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { Input } from "../components/Input.tsx";
import { PasswordInput } from "../components/PasswordInput.tsx";
import { getQueryClient } from "../utils/query_client.ts";

function SignInFormContent() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [formError, setFormError] = useState("");

  const signInMutation = useSignIn();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setFormError("");

    signInMutation.mutate(
      { email, password },
      {
        onSuccess: (data) => {
          if ("token" in data) {
            globalThis.location.href = "/";
          } else if ("message" in data) {
            setFormError((data as { message: string }).message);
          }
        },
        onError: (error) => {
          setFormError(
            (error as Error).message || "An unexpected error occurred",
          );
        },
      },
    );
  };

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

      <PasswordInput
        id="password"
        label="Password"
        value={password}
        onInput={(e) => setPassword(e.currentTarget.value)}
        required
      />

      {formError && (
        <div class="rounded-md bg-red-50 p-4">
          <div class="flex">
            <div class="ml-3">
              <h3 class="text-sm font-medium text-red-800">Error</h3>
              <div class="mt-2 text-sm text-red-700">
                <p>{formError}</p>
              </div>
            </div>
          </div>
        </div>
      )}

      <div>
        <Button
          type="submit"
          className="w-full flex justify-center"
          isLoading={signInMutation.isPending}
        >
          Sign in
        </Button>
      </div>
    </form>
  );
}

export default function SignInForm() {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <SignInFormContent />
    </QueryClientProvider>
  );
}
