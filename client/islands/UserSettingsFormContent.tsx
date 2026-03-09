import { useEffect, useState } from "preact/hooks";
import {
  useMe,
  useResendVerificationEmail,
  useUpdateUser,
} from "../services/auth/hooks.ts";
import { Button } from "../components/Button.tsx";
import { Input } from "../components/Input.tsx";

export default function UserSettingsFormContent() {
  const { data: user, isLoading } = useMe();
  const [email, setEmail] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [bio, setBio] = useState("");
  const [message, setMessage] = useState<
    { text: string; type: "success" | "error" } | null
  >(null);

  useEffect(() => {
    if (user) {
      if (user.email) setEmail(user.email);
      if (user.displayName) setDisplayName(user.displayName);
      if (user.bio) setBio(user.bio);
    }
  }, [user?.email, user?.displayName, user?.bio]);

  const updateUserMutation = useUpdateUser();
  const resendMutation = useResendVerificationEmail();

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    setMessage(null);
    updateUserMutation.mutate(
      {
        email,
        displayName: displayName || undefined,
        bio: bio || undefined,
      },
      {
        onSuccess: (data) => {
          if ("email" in data) {
            setMessage({
              text: "Profile updated successfully",
              type: "success",
            });
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
    <div class="space-y-6">
      {user && !user.emailVerifiedAt && (
        <div class="rounded-md bg-yellow-50 p-4">
          <div class="flex items-start justify-between">
            <div>
              <h3 class="text-sm font-medium text-yellow-800">
                Email not verified
              </h3>
              <p class="mt-1 text-sm text-yellow-700">
                Verify your email to create and edit posts.
              </p>
            </div>
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
        </div>
      )}
      <form onSubmit={handleSubmit} class="space-y-4">
        <Input
          id="email"
          label="Email"
          type="email"
          value={email}
          onInput={(e) => setEmail(e.currentTarget.value)}
          required
        />
        <Input
          id="displayName"
          label="Display Name"
          type="text"
          value={displayName}
          onInput={(e) => setDisplayName(e.currentTarget.value)}
        />
        <div>
          <label class="block text-sm font-medium text-gray-700" for="bio">
            Bio
          </label>
          <textarea
            id="bio"
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm border px-3 py-2"
            rows={3}
            value={bio}
            onInput={(e) => setBio((e.target as HTMLTextAreaElement).value)}
          />
        </div>

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
                    message.type === "success"
                      ? "text-green-800"
                      : "text-red-800"
                  }`}
                >
                  {message.type === "success" ? "Success" : "Error"}
                </h3>
                <div
                  class={`mt-2 text-sm ${
                    message.type === "success"
                      ? "text-green-700"
                      : "text-red-700"
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
    </div>
  );
}
