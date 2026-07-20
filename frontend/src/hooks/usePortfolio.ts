import { useEffect, useState } from "react";
import { useAccount } from "wagmi";
import { api, mapBackendPortfolio } from "@/lib/api";
import { demoPortfolio } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Portfolio } from "@/types/app";

export function usePortfolio() {
  const { address } = useAccount();
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Portfolio | null>(() =>
    isAuthenticated ? null : demoPortfolio
  );
  const [isLoading, setIsLoading] = useState(isAuthenticated);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  useEffect(() => {
    if (!isAuthenticated) {
      setData(demoPortfolio);
      setError(null);
      setIsLoading(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    api.portfolio
      .get()
      .then((res) => {
        if (cancelled) return;
        setData(mapBackendPortfolio(res));
        setError(null);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [address, isAuthenticated, reloadKey]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch, isDemo };
}
