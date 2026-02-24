import { useEffect, useState } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { useVerifyEmail } from "../services/auth/hooks.ts";
import { getQueryClient } from "../utils/query_client.ts";

interface Props {
  token: string;
}

type Status = "loading" | "success" | "error";

function VerifyEmailHandlerContent({ token }: Props) {
  const mutation = useVerifyEmail();
  const [status, setStatus] = useState<Status>("loading");
  const [message, setMessage] = useState("");

  useEffect(() => {
    if (!token) {
      setStatus("error");
      setMessage("Invalid or missing verification token.");
      return;
    }
    mutation.mutate(token, {
      onSuccess: (data) => {
        const d = data as { __typename?: string; message?: string };
        if (d.__typename === "EmailVerifySuccess") {
          setStatus("success");
        } else {
          setStatus("error");
          setMessage(d.message || "Verification failed. The link may have expired.");
        }
      },
      onError: (err) => {
        setStatus("error");
        setMessage((err as Error).message || "Verification failed.");
      },
    });
  }, [token]);

  if (status === "loading") {
    return <p class="text-sm text-gray-500">Verifying your email...</p>;
  }

  if (status === "error") {
    return (
      <div class="rounded-md bg-red-50 p-4">
        <p class="text-sm text-red-700">{message}</p>
        <p class="mt-2 text-sm">
          <a href="/settings" class="text-blue-600 hover:text-blue-500">
            {message.includes("missing") ? "Go to settings to resend" : "Resend verification email"}
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

export default function VerifyEmailHandler({ token }: Props) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <VerifyEmailHandlerContent token={token} />
    </QueryClientProvider>
  );
}
