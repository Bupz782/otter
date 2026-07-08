import { useEffect, useState } from "react";
import { api, mapBackendAgent } from "@/lib/api";
import type { Agent } from "@/types/app";

export function useAgent(id: string | undefined) {
  const [data, setData] = useState<Agent | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!id) {
      setIsLoading(false);
      return;
    }
    setIsLoading(true);
    api.agents
      .get(id)
      .then((res) => setData(mapBackendAgent(res)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, [id]);

  return { data, isLoading, error };
}
