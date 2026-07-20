import { useEffect, useState } from "react";
import { AUTH_TOKEN_CHANGED_EVENT, loadAuthToken } from "@/lib/api";

// Tracks whether an auth token exists. Updates on sign-in/sign-out in this
// tab (AUTH_TOKEN_CHANGED_EVENT) and on cross-tab changes (storage event).
export function useAuthToken() {
  const [token, setToken] = useState<string | null>(() => loadAuthToken());

  useEffect(() => {
    const refresh = () => setToken(loadAuthToken());
    const onStorage = (event: StorageEvent) => {
      if (event.key === null || event.key === "otter_token") refresh();
    };
    window.addEventListener("storage", onStorage);
    window.addEventListener(AUTH_TOKEN_CHANGED_EVENT, refresh);
    return () => {
      window.removeEventListener("storage", onStorage);
      window.removeEventListener(AUTH_TOKEN_CHANGED_EVENT, refresh);
    };
  }, []);

  return { token, isAuthenticated: token !== null };
}
