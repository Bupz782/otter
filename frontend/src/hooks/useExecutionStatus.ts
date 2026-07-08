import { useEffect, useState } from "react";
import { getExecutionStatus } from "@/lib/mock-api";
import type { ExecutionStatus } from "@/types/app";

export function useExecutionStatus(intentId: string | undefined, poll = false) {
  const [data, setData] = useState<ExecutionStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!intentId) {
      setIsLoading(false);
      return;
    }
    let mounted = true;
    const fetch = () => {
      getExecutionStatus(intentId)
        .then((result) => {
          if (mounted) setData(result);
        })
        .catch((err) => {
          if (mounted) setError(err);
        })
        .finally(() => {
          if (mounted) setIsLoading(false);
        });
    };
    fetch();
    if (!poll)
      return () => {
        mounted = false;
      };
    const interval = setInterval(fetch, 5000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [intentId, poll]);

  return { data, isLoading, error };
}
