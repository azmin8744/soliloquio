import { Head } from "$fresh/runtime.ts";
import UserSettingsForm from "../islands/UserSettingsForm.tsx";
import { Card } from "../components/Card.tsx";

export default function Settings() {
  return (
    <>
      <Head>
        <title>Settings - Soliloquio</title>
      </Head>
      <div class="flex-1 flex items-center justify-center overflow-y-auto">
        <div class="w-full max-w-md p-8">
          <Card>
            <UserSettingsForm />
          </Card>
        </div>
      </div>
    </>
  );
}
