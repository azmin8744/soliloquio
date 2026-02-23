import { Head } from "$fresh/runtime.ts";
import type { RouteConfig } from "$fresh/server.ts";
import SignInForm from "../../islands/SignInForm.tsx";
import { Card } from "../../components/Card.tsx";

export const config: RouteConfig = { skipInheritedLayouts: true };

export default function SignIn() {
  return (
    <>
      <Head>
        <title>Sign In - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Sign in to your account
          </h2>
          <p class="mt-2 text-center text-sm text-gray-600">
            Or{" "}
            <a
              href="/auth/signup"
              class="font-medium text-blue-600 hover:text-blue-500"
            >
              create a new account
            </a>
          </p>
        </div>

        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <SignInForm />
          </Card>
        </div>
      </div>
    </>
  );
}
