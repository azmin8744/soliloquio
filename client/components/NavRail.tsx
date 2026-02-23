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
              <a
                href="/settings"
                class="text-gray-400 hover:text-white transition-colors"
                title="Settings"
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
                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                </svg>
              </a>
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
