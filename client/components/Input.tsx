import { JSX } from "preact";

interface InputProps extends JSX.HTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  id: string;
}

export function Input({ label, error, id, className, ...props }: InputProps) {
  return (
    <div class="space-y-1">
      {label && (
        <label htmlFor={id} class="block text-sm font-medium text-gray-700">
          {label}
        </label>
      )}
      <input
        id={id}
        {...props}
        class={`block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm ${
          error
            ? "border-red-300 text-red-900 placeholder-red-300 focus:ring-red-500 focus:border-red-500"
            : ""
        } ${className || ""}`}
      />
      {error && <p class="mt-2 text-sm text-red-600">{error}</p>}
    </div>
  );
}
