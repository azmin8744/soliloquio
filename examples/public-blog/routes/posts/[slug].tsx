import { Head } from "$fresh/runtime.ts";
import { Handlers, PageProps } from "$fresh/server.ts";
import { getPost } from "../../services/posts/api.ts";
import { PublicPost } from "../../services/posts/types.ts";
import PostNav from "../../components/PostNav.tsx";

interface Data {
  post: PublicPost;
  siteTitle: string;
  siteUrl: string;
}

export const handler: Handlers<Data> = {
  async GET(_req, ctx) {
    const post = await getPost(ctx.params.slug);
    if (!post) return ctx.renderNotFound();

    return ctx.render({
      post,
      siteTitle: Deno.env.get("SITE_TITLE") ?? "Blog",
      siteUrl: Deno.env.get("SITE_URL") ?? "",
    });
  },
};

function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

export default function PostPage({ data }: PageProps<Data>) {
  const { post, siteTitle, siteUrl } = data;
  const postUrl = siteUrl && post.slug
    ? `${siteUrl}/posts/${post.slug}`
    : undefined;

  return (
    <>
      <Head>
        <title>{post.title} - {siteTitle}</title>
        {post.description && (
          <meta name="description" content={post.description} />
        )}
        <meta property="og:title" content={post.title} />
        {post.description && (
          <meta property="og:description" content={post.description} />
        )}
        <meta property="og:type" content="article" />
        {postUrl && <meta property="og:url" content={postUrl} />}
        {post.cover_image && (
          <meta property="og:image" content={post.cover_image} />
        )}
        {post.first_published_at && (
          <meta
            property="article:published_time"
            content={post.first_published_at}
          />
        )}
      </Head>
      <main class="max-w-2xl mx-auto px-4 py-12">
        <a href="/" class="text-sm text-gray-500 hover:text-gray-700">
          ← All posts
        </a>
        <article class="mt-6">
          {post.cover_image && (
            <img
              src={post.cover_image}
              alt={post.title}
              class="w-full rounded-lg mb-8 object-cover max-h-80"
            />
          )}
          <p class="text-sm text-gray-500 mb-2">
            {formatDate(post.first_published_at)}
            {post.author.display_name && (
              <span> · {post.author.display_name}</span>
            )}
          </p>
          <h1 class="text-3xl font-bold mb-4">{post.title}</h1>
          <div
            class="prose prose-gray max-w-none"
            dangerouslySetInnerHTML={{ __html: post.content }}
          />
        </article>
        <PostNav prev={post.prev_post} next={post.next_post} />
      </main>
    </>
  );
}
