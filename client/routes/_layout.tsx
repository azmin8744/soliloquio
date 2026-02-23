import type { LayoutProps } from "$fresh/server.ts";
import NavRailIsland from "../islands/NavRailIsland.tsx";

export default function AppLayout({ Component, url }: LayoutProps) {
  const activePage = url.pathname === "/settings" ? "settings" : "posts";
  return (
    <div class="h-screen flex bg-gray-50">
      <NavRailIsland activePage={activePage} />
      <Component />
    </div>
  );
}
