import { Head } from "$fresh/runtime.ts";
import type { PageProps, RouteConfig } from "$fresh/server.ts";
import ResetPasswordForm from "../../islands/ResetPasswordForm.tsx";
import { Card } from "../../components/Card.tsx";

export const config: RouteConfig = { skipInheritedLayouts: true };

export default function ResetPassword(props: PageProps) {
  const token = props.url.searchParams.get("token") ?? "";
  return (
    <>
      <Head>
        <title>Reset Password - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Set a new password
          </h2>
        </div>
        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <ResetPasswordForm token={token} />
          </Card>
        </div>
      </div>
    </>
  );
}
