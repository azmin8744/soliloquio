import { Head } from "$fresh/runtime.ts";
import SignUpForm from "@/islands/SignUpForm.tsx";
import { Card } from "@/components/Card.tsx";

export default function SignUp() {
  return (
    <>
      <Head>
        <title>Sign Up - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Create your account
          </h2>
          <p class="mt-2 text-center text-sm text-gray-600">
            Or{" "}
            <a
              href="/auth/signin"
              class="font-medium text-blue-600 hover:text-blue-500"
            >
              sign in to your existing account
            </a>
          </p>
        </div>

        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <SignUpForm />
          </Card>
        </div>
      </div>
    </>
  );
}
