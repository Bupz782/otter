import { useEffect, useState } from "react";
import { api, mapBackendLeaderboardEntry } from "@/lib/api";
import type { LeaderboardEntry } from "@/types/app";

export function useLeaderboard() {
  const [data, setData] = useState<LeaderboardEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.leaderboard
      .get()
      .then((res) => setData(res.entries.map(mapBackendLeaderboardEntry)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
