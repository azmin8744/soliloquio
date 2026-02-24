import { Head } from "$fresh/runtime.ts";
import type { PageProps, RouteConfig } from "$fresh/server.ts";
import VerifyEmailHandler from "../../islands/VerifyEmailHandler.tsx";
import { Card } from "../../components/Card.tsx";

export const config: RouteConfig = { skipInheritedLayouts: true };

export default function VerifyEmail(props: PageProps) {
  const token = props.url.searchParams.get("token") ?? "";
  return (
    <>
      <Head>
        <title>Verify Email - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Email verification
          </h2>
        </div>
        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <VerifyEmailHandler token={token} />
          </Card>
        </div>
      </div>
    </>
  );
}
