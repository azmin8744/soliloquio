import { useEffect } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useVerifyEmail } from "../services/auth/hooks.ts";
import { getQueryClient } from "../utils/query_client.ts";

interface Props {
  token: string;
}

function VerifyEmailHandlerContent({ token }: Props) {
  const mutation = useVerifyEmail();

  useEffect(() => {
    if (token) mutation.mutate(token);
  }, [token]);

  if (!token) {
    return (
      <div class="rounded-md bg-red-50 p-4">
        <p class="text-sm text-red-700">
          Invalid or missing verification token.
        </p>
        <p class="mt-2 text-sm">
          <a href="/settings" class="text-blue-600 hover:text-blue-500">
            Go to settings to resend
          </a>
        </p>
      </div>
    );
  }

  if (mutation.isPending) {
    return <p class="text-sm text-gray-500">Verifying your email...</p>;
  }

  if (
    mutation.isError ||
    (mutation.data && "message" in mutation.data &&
      !("message" in (mutation.data as { message: string })))
  ) {
    return (
      <div class="rounded-md bg-red-50 p-4">
        <p class="text-sm text-red-700">
          {(mutation.error as Error)?.message ||
            "Verification failed. The link may have expired."}
        </p>
        <p class="mt-2 text-sm">
          <a href="/settings" class="text-blue-600 hover:text-blue-500">
            Resend verification email
          </a>
        </p>
      </div>
    );
  }

  if (mutation.isSuccess) {
    const data = mutation.data as { message?: string };
    const isError = !data?.message || data.message.includes("Invalid") ||
      data.message.includes("expired");

    if (isError) {
      return (
        <div class="rounded-md bg-red-50 p-4">
          <p class="text-sm text-red-700">
            Verification failed. The link may have expired.
          </p>
          <p class="mt-2 text-sm">
            <a href="/settings" class="text-blue-600 hover:text-blue-500">
              Resend verification email
            </a>
          </p>
        </div>
      );
    }

    return (
      <div class="rounded-md bg-green-50 p-4">
        <p class="text-sm text-green-800">Email verified successfully!</p>
        <p class="mt-2 text-sm">
          <a href="/" class="text-blue-600 hover:text-blue-500">
            Go to dashboard
          </a>
        </p>
      </div>
    );
  }

  return null;
}

export default function VerifyEmailHandler({ token }: Props) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <VerifyEmailHandlerContent token={token} />
    </QueryClientProvider>
  );
}
