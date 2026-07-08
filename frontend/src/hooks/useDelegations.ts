import { useEffect, useState } from "react";
import { api, mapBackendDelegation } from "@/lib/api";
import type { Delegation } from "@/types/app";

export function useDelegations() {
  const [data, setData] = useState<Delegation[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.delegations
      .list()
      .then((res) => {
        setData(res.delegations.map(mapBackendDelegation));
        setError(null);
      })
      .catch((err) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
