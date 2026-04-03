import { PrevNextPost } from "../services/posts/types.ts";

interface Props {
  prev: PrevNextPost | null;
  next: PrevNextPost | null;
}

function postHref(p: PrevNextPost): string {
  return p.slug ? `/posts/${p.slug}` : `/posts/${p.id}`;
}

export default function PostNav({ prev, next }: Props) {
  if (!prev && !next) return null;
  return (
    <nav class="mt-12 pt-6 border-t border-gray-200 flex justify-between gap-4">
      <div>
        {prev && (
          <a href={postHref(prev)} class="group flex flex-col">
            <span class="text-xs text-gray-500 mb-1">← Older</span>
            <span class="text-sm font-medium text-gray-800 group-hover:text-blue-600">
              {prev.title}
            </span>
          </a>
        )}
      </div>
      <div class="text-right">
        {next && (
          <a href={postHref(next)} class="group flex flex-col items-end">
            <span class="text-xs text-gray-500 mb-1">Newer →</span>
            <span class="text-sm font-medium text-gray-800 group-hover:text-blue-600">
              {next.title}
            </span>
          </a>
        )}
      </div>
    </nav>
  );
}
