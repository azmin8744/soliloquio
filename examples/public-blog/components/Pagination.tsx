import { PageInfo } from "../services/posts/types.ts";

interface Props {
  pageInfo: PageInfo;
  currentPage: number;
  totalPages: number | null;
}

export default function Pagination({ pageInfo, currentPage, totalPages }: Props) {
  const hasPrev = currentPage > 1;
  const hasNext = pageInfo.hasNextPage;
  if (!hasPrev && !hasNext) return null;

  return (
    <nav class="mt-10 flex items-center justify-between gap-4 border-t border-gray-200 pt-6">
      <div class="flex-1">
        {hasPrev && (
          <a
            href={currentPage === 2 ? "/" : `/?page=${currentPage - 1}`}
            class="px-4 py-2 rounded-md border border-gray-300 text-sm text-gray-700 hover:bg-gray-50"
          >
            ← Previous
          </a>
        )}
      </div>
      <div class="flex-1 text-center">
        <span class="text-sm text-gray-500">
          {totalPages != null
            ? `Page ${currentPage} of ${totalPages}`
            : `Page ${currentPage}`}
        </span>
      </div>
      <div class="flex-1 flex justify-end">
        {hasNext && (
          <a
            href={`/?page=${currentPage + 1}`}
            class="px-4 py-2 rounded-md border border-gray-300 text-sm text-gray-700 hover:bg-gray-50"
          >
            Next →
          </a>
        )}
      </div>
    </nav>
  );
}
