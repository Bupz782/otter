import { useEffect, useState } from "react";
import { api, mapBackendStrategy } from "@/lib/api";
import type { Strategy } from "@/types/app";

export function useStrategies() {
  const [data, setData] = useState<Strategy[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.strategies
      .list()
      .then((res) => setData(res.strategies.map(mapBackendStrategy)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
