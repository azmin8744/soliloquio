import type { LayoutProps } from "$fresh/server.ts";
import NavRailIsland from "../islands/NavRailIsland.tsx";

export default function AppLayout({ Component, url }: LayoutProps) {
  const activePage = url.pathname === "/settings"
    ? "settings"
    : url.pathname === "/assets"
    ? "assets"
    : "posts";
  return (
    <div class="h-screen flex bg-gray-50">
      <NavRailIsland activePage={activePage} />
      <div class="flex-1 min-h-0 flex flex-col overflow-hidden pb-14 md:pb-0">
        <Component />
      </div>
    </div>
  );
}
