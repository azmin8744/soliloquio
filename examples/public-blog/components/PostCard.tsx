import { PublicPost } from "../services/posts/types.ts";

interface Props {
  post: PublicPost;
}

function excerpt(post: PublicPost): string {
  const source = post.description ||
    post.content.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
  return source.length > 240 ? source.slice(0, 240).trimEnd() + "…" : source;
}

function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

export default function PostCard({ post }: Props) {
  const href = post.slug ? `/posts/${post.slug}` : `/posts/${post.id}`;
  return (
    <article class="border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition-shadow">
      {post.cover_image && (
        <a href={href}>
          <img
            src={post.cover_image}
            alt={post.title}
            class="w-full h-48 object-cover"
          />
        </a>
      )}
      <div class="p-5">
        <p class="text-sm text-gray-500 mb-1">
          {formatDate(post.first_published_at)}
        </p>
        <h2 class="text-xl font-semibold mb-2">
          <a href={href} class="hover:text-blue-600">
            {post.title}
          </a>
        </h2>
        <p class="text-gray-600 text-sm mt-2">{excerpt(post)}</p>
        <a
          href={href}
          class="inline-block mt-3 text-sm text-blue-600 hover:underline"
        >
          Read more →
        </a>
      </div>
    </article>
  );
}
