import type { ComponentChildren } from "preact";
import type { User } from "../domains/users.ts";

// --- Atom: style utility ---

function NavRailPanel(active?: boolean): string {
  return active
    ? "p-2 rounded-lg bg-gray-700 text-white"
    : "p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors";
}

// --- Molecules ---

interface NavRailLinkProps {
  href: string;
  active?: boolean;
  title?: string;
  children: ComponentChildren;
}

export function NavRailLink({ href, active, title, children }: NavRailLinkProps) {
  if (active) {
    return (
      <span class={NavRailPanel(true)} title={title}>{children}</span>
    );
  }
  return (
    <a href={href} class={NavRailPanel()} title={title}>{children}</a>
  );
}

interface NavRailButtonProps {
  onClick: () => void;
  disabled?: boolean;
  title?: string;
  children: ComponentChildren;
}

export function NavRailButton(
  { onClick, disabled, title, children }: NavRailButtonProps,
) {
  return (
    <button
      type="button"
      class={NavRailPanel()}
      onClick={onClick}
      disabled={disabled}
      title={title}
    >
      {children}
    </button>
  );
}

// --- NavRail ---

const PostsIcon = () => (
  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
    />
  </svg>
);

const GearIcon = () => (
  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
);

const LogoutIcon = () => (
  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
    />
  </svg>
);

interface NavRailProps {
  user: User | null | undefined;
  isLoading: boolean;
  onLogout: () => void;
  isLoggingOut: boolean;
  activePage: "posts" | "settings";
}

export function NavRail(
  { user, isLoading, onLogout, isLoggingOut, activePage }: NavRailProps,
) {
  return (
    <div class="w-16 bg-gray-900 flex flex-col items-center py-4 flex-shrink-0">
      <div class="flex flex-col items-center gap-1">
        <div class="text-white font-bold text-lg mb-2" title="Soliloquio">S</div>
        <NavRailLink href="/posts" active={activePage === "posts"} title="Posts">
          <PostsIcon />
        </NavRailLink>
      </div>

      <div class="flex flex-col items-center gap-1 mt-auto">
        {isLoading
          ? <div class="w-8 h-8 bg-gray-700 rounded-full animate-pulse" />
          : user
          ? (
            <>
              <NavRailLink
                href="/settings"
                active={activePage === "settings"}
                title="Settings"
              >
                <GearIcon />
              </NavRailLink>
              <div
                class="w-8 h-8 bg-indigo-500 rounded-full flex items-center justify-center text-white text-xs font-medium my-1"
                title={user.email}
              >
                {user.email[0].toUpperCase()}
              </div>
              <NavRailButton
                onClick={onLogout}
                disabled={isLoggingOut}
                title="Sign out"
              >
                <LogoutIcon />
              </NavRailButton>
            </>
          )
          : null}
      </div>
    </div>
  );
}
