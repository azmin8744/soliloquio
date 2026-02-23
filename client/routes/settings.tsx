import { Head } from "$fresh/runtime.ts";
import UserSettingsForm from "../islands/UserSettingsForm.tsx";
import { Card } from "../components/Card.tsx";

export default function Settings() {
  return (
    <>
      <Head>
        <title>Settings - Soliloquio</title>
      </Head>
      <div class="min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8">
        <div class="sm:mx-auto sm:w-full sm:max-w-md">
          <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
            Settings
          </h2>
        </div>

        <div class="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
          <Card>
            <UserSettingsForm />
          </Card>
        </div>
      </div>
    </>
  );
}
