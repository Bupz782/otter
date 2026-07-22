import { useEffect, useState } from "react";
import { api, mapBackendProof } from "@/lib/api";
import { demoProofs } from "@/lib/demo-data";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { Proof } from "@/types/app";

export function useProofs() {
  const { isAuthenticated } = useAuthToken();
  const isDemo = !isAuthenticated;
  const [data, setData] = useState<Proof[]>(() => (isAuthenticated ? [] : demoProofs));
  const [isLoading, setIsLoading] = useState(isAuthenticated);
  const [error, setError] = useState<Error | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  // True when the API itself flags the payload as demonstration data (A2).
  const [isBackendDemo, setIsBackendDemo] = useState(false);

  useEffect(() => {
    if (!isAuthenticated) {
      setData(demoProofs);
      setError(null);
      setIsLoading(false);
      setIsBackendDemo(false);
      return;
    }
    let cancelled = false;
    setIsLoading(true);
    api.proofs
      .list()
      .then((res) => {
        if (cancelled) return;
        setData(res.proofs.map(mapBackendProof));
        setIsBackendDemo(res.demo === true);
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
  }, [isAuthenticated, reloadKey]);

  const refetch = () => setReloadKey((key) => key + 1);

  return { data, isLoading, error, refetch, isDemo, isBackendDemo };
}
