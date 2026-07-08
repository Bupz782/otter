import { useEffect, useState } from "react";
import { api, mapBackendAgent } from "@/lib/api";
import type { Agent } from "@/types/app";

export function useAgents() {
  const [data, setData] = useState<Agent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.agents
      .list()
      .then((res) => setData(res.agents.map(mapBackendAgent)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
