import { JSX } from "preact";

interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  title?: string;
}

export function Card({ title, children, className, ...props }: CardProps) {
  return (
    <div
      class={`bg-white shadow rounded-lg overflow-hidden ${className || ""}`}
      {...props}
    >
      {title && (
        <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
          <h3 class="text-lg leading-6 font-medium text-gray-900">{title}</h3>
        </div>
      )}
      <div class="px-4 py-5 sm:p-6">{children}</div>
    </div>
  );
}
