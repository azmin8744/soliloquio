import type { UseMutationResult } from "@tanstack/react-query";

interface EmailVerificationBannerProps {
  resendMutation: UseMutationResult<unknown, Error, void>;
}

export function EmailVerificationBanner(
  { resendMutation }: EmailVerificationBannerProps,
) {
  return (
    <div class="bg-yellow-50 border-b border-yellow-200 px-4 py-2 flex items-center justify-between flex-shrink-0">
      <p class="text-sm text-yellow-800">
        Verify your email to create and manage posts.{" "}
        <a href="/settings" class="font-medium underline hover:text-yellow-700">
          Go to settings
        </a>
      </p>
      <button
        type="button"
        class="ml-4 text-sm font-medium text-yellow-800 underline hover:text-yellow-700 disabled:opacity-50"
        onClick={() => resendMutation.mutate()}
        disabled={resendMutation.isPending || resendMutation.isSuccess}
      >
        {resendMutation.isPending
          ? "Sending..."
          : resendMutation.isSuccess
          ? "Sent!"
          : "Resend email"}
      </button>
    </div>
  );
}
