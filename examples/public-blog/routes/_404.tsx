import { Head } from "$fresh/runtime.ts";

export default function NotFound() {
  return (
    <>
      <Head>
        <title>Post not found</title>
      </Head>
      <div class="min-h-screen flex items-center justify-center">
        <div class="text-center">
          <p class="text-gray-500 mb-4">Post not found.</p>
          <a href="/" class="text-blue-600 hover:underline">← Back to posts</a>
        </div>
      </div>
    </>
  );
}
