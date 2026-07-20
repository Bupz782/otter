import { useEffect, useState } from "react";
import { api, mapBackendStrategyDetail } from "@/lib/api";
import type { Strategy } from "@/types/app";

export function useStrategy(id: string | undefined) {
  const [data, setData] = useState<Strategy | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!id) return;
    setIsLoading(true);
    api.strategies
      .get(id)
      .then((res) => setData(mapBackendStrategyDetail(res)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, [id]);

  return { data, isLoading, error };
}
