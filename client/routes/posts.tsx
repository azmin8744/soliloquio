import { Head } from "$fresh/runtime.ts";
import Workspace from "../islands/Workspace.tsx";

export default function Posts() {
  return (
    <>
      <Head>
        <title>Soliloquio</title>
      </Head>
      <Workspace />
    </>
  );
}
