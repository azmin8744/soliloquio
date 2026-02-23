import { useCallback, useEffect } from "preact/hooks";
import { QueryClientProvider } from "@tanstack/react-query";
import { getQueryClient } from "../utils/query_client.ts";
import { useLogout, useMe } from "../services/auth/hooks.ts";
import { NavRail } from "../components/NavRail.tsx";

interface NavRailIslandProps {
  activePage: "posts" | "settings";
}

function NavRailIslandInner({ activePage }: NavRailIslandProps) {
  const { data: user, isLoading } = useMe();
  const logout = useLogout();

  useEffect(() => {
    if (!isLoading && !user) {
      globalThis.location.href = "/auth/signin";
    }
  }, [isLoading, user]);

  const handleLogout = useCallback(() => {
    logout.mutate(undefined, {
      onSuccess: () => {
        globalThis.location.href = "/auth/signin";
      },
    });
  }, []);

  return (
    <NavRail
      user={user}
      isLoading={isLoading}
      onLogout={handleLogout}
      isLoggingOut={logout.isPending}
      activePage={activePage}
    />
  );
}

export default function NavRailIsland({ activePage }: NavRailIslandProps) {
  return (
    <QueryClientProvider client={getQueryClient()}>
      <NavRailIslandInner activePage={activePage} />
    </QueryClientProvider>
  );
}
