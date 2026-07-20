import { useEffect, useState } from "react";
import { api, type BackendExecutionRecord } from "@/lib/api";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { ExecutionStatus, ExecutionStep } from "@/types/app";

function toStep(record: BackendExecutionRecord): ExecutionStep {
  return {
    status: "confirmed",
    label: "Executed",
    detail:
      record.gas_used > 0
        ? `Transaction confirmed · ${record.gas_used.toLocaleString()} gas`
        : "Transaction confirmed on-chain",
    timestamp: new Date(record.created_at * 1000).toISOString(),
  };
}

/**
 * Real execution history for an intent, from GET /api/v1/executions filtered
 * client-side by intent_id (the backend endpoint has no per-intent filter;
 * list_executions in crates/interfaces/src/bin/metis_api.rs). The backend only
 * writes a record once a transaction confirms, so an empty result means "no
 * execution yet": the hook returns null and the page shows an honest static
 * state built from the intent's own status.
 */
export function useExecutionStatus(intentId: string | undefined, poll = false) {
  const { isAuthenticated } = useAuthToken();
  const [data, setData] = useState<ExecutionStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!intentId || !isAuthenticated) {
      setData(null);
      setError(null);
      setIsLoading(false);
      return;
    }
    let mounted = true;
    const fetchStatus = () => {
      api.executions
        .list()
        .then((res) => {
          if (!mounted) return;
          const records = res.executions
            .filter((record) => record.intent_id === intentId)
            .sort((a, b) => a.created_at - b.created_at);
          if (records.length === 0) {
            setData(null);
          } else {
            const steps = records.map(toStep);
            setData({
              intentId,
              currentStep: "confirmed",
              steps,
              startedAt: steps[0].timestamp ?? "",
              updatedAt: steps[steps.length - 1].timestamp ?? "",
            });
          }
          setError(null);
        })
        .catch((err) => {
          if (mounted) setError(err instanceof Error ? err : new Error(String(err)));
        })
        .finally(() => {
          if (mounted) setIsLoading(false);
        });
    };
    setIsLoading(true);
    fetchStatus();
    if (!poll)
      return () => {
        mounted = false;
      };
    const interval = setInterval(fetchStatus, 5000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [intentId, poll, isAuthenticated]);

  return { data, isLoading, error };
}
