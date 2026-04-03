import { Head } from "$fresh/runtime.ts";
import { Handlers, PageProps } from "$fresh/server.ts";
import { getAuthor, getPosts } from "../services/posts/api.ts";
import { PageInfo, PublicAuthor, PublicPost } from "../services/posts/types.ts";
import SiteTitle from "../components/SiteTitle.tsx";
import PostCard from "../components/PostCard.tsx";
import Pagination from "../components/Pagination.tsx";

interface Data {
  posts: PublicPost[];
  pageInfo: PageInfo;
  author: PublicAuthor;
  siteTitle: string;
  siteUrl: string;
  currentPage: number;
  totalPages: number | null;
}

export const handler: Handlers<Data> = {
  async GET(req, ctx) {
    const url = new URL(req.url);
    const page = Math.max(1, parseInt(url.searchParams.get("page") ?? "1", 10));
    const first = parseInt(Deno.env.get("POSTS_PER_PAGE") ?? "10", 10);
    const siteTitle = Deno.env.get("SITE_TITLE") ?? "Blog";
    const siteUrl = Deno.env.get("SITE_URL") ?? "";

    const [connection, author] = await Promise.all([
      getPosts({ page, first }),
      getAuthor(),
    ]);

    return ctx.render({
      posts: connection.nodes,
      pageInfo: connection.pageInfo,
      author,
      siteTitle,
      siteUrl,
      currentPage: page,
      totalPages: connection.totalPages,
    });
  },
};

export default function Index({ data }: PageProps<Data>) {
  const { posts, pageInfo, author, siteTitle, siteUrl, currentPage, totalPages } = data;
  const description = author.bio ??
    (author.display_name ? `${author.display_name}'s blog` : siteTitle);

  return (
    <>
      <Head>
        <title>{siteTitle}</title>
        <meta name="description" content={description} />
        <meta property="og:title" content={siteTitle} />
        <meta property="og:description" content={description} />
        <meta property="og:type" content="website" />
        {siteUrl && <meta property="og:url" content={siteUrl} />}
      </Head>
      <main class="max-w-3xl mx-auto px-4 py-12">
        <SiteTitle title={siteTitle} />
        {posts.length === 0
          ? <p class="text-gray-500">No posts yet.</p>
          : (
            <div class="grid gap-6">
              {posts.map((post) => <PostCard key={post.id} post={post} />)}
            </div>
          )}
        <Pagination pageInfo={pageInfo} currentPage={currentPage} totalPages={totalPages} />
      </main>
    </>
  );
}
