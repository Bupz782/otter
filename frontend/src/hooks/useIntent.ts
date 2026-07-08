import { useEffect, useState } from "react";
import { api, mapBackendIntent } from "@/lib/api";
import type { Intent } from "@/types/app";

export function useIntent(id: string | undefined) {
  const [data, setData] = useState<Intent | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }
    setIsLoading(true);
    api.intents
      .get(id)
      .then((record) => setData(mapBackendIntent(record)))
      .catch((err) => setError(err instanceof Error ? err : new Error(String(err))))
      .finally(() => setIsLoading(false));
  }, [id]);

  return { data, isLoading, error };
}
