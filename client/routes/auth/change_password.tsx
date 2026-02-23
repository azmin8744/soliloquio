import { Head } from "$fresh/runtime.ts";
import type { RouteConfig } from "$fresh/server.ts";
import ChangePasswordForm from "../../islands/ChangePasswordForm.tsx";
import { Card } from "../../components/Card.tsx";

export const config: RouteConfig = { skipInheritedLayouts: true };

export default function ChangePassword() {
  return (
    <>
      <Head>
        <title>Change Password - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Change Password
          </h2>
        </div>

        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <ChangePasswordForm />
          </Card>
        </div>
      </div>
    </>
  );
}
