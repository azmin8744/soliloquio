import { Head } from "$fresh/runtime.ts";
import AssetsLibrary from "@/islands/AssetsLibrary.tsx";

export default function Assets() {
  return (
    <>
      <Head>
        <title>Assets - Soliloquio</title>
      </Head>
      <AssetsLibrary />
    </>
  );
}
