import type { User } from "../domains/users.ts";
import { Button } from "./Button.tsx";

interface NavRailProps {
  user: User | null | undefined;
  isLoading: boolean;
  onLogout: () => void;
  isLoggingOut: boolean;
}

export function NavRail(
  { user, isLoading, onLogout, isLoggingOut }: NavRailProps,
) {
  return (
    <div class="w-16 bg-gray-900 flex flex-col items-center justify-between py-4 flex-shrink-0">
      {/* Logo */}
      <div class="text-white font-bold text-lg" title="Soliloquio">S</div>

      {/* User + Logout */}
      <div class="flex flex-col items-center gap-2">
        {isLoading
          ? <div class="w-8 h-8 bg-gray-700 rounded-full animate-pulse" />
          : user
          ? (
            <>
              <div
                class="w-8 h-8 bg-indigo-500 rounded-full flex items-center justify-center text-white text-xs font-medium"
                title={user.email}
              >
                {user.email[0].toUpperCase()}
              </div>
              <button
                onClick={onLogout}
                disabled={isLoggingOut}
                class="text-gray-400 hover:text-white transition-colors"
                title="Sign out"
              >
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                  />
                </svg>
              </button>
            </>
          )
          : null}
      </div>
    </div>
  );
}
