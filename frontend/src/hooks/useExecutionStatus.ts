import { useEffect, useState } from "react";
import { api, type BackendExecutionRecord, type BackendIntentEvent } from "@/lib/api";
import { useAuthToken } from "@/hooks/useAuthToken";
import type { ExecutionStatus, ExecutionStep } from "@/types/app";

function toExecutionStep(record: BackendExecutionRecord): ExecutionStep {
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

const EVENT_LABEL: Record<BackendIntentEvent["kind"], string> = {
  parsed: "Parsed & registered",
  condition_met: "Condition met",
  proof_started: "ZK proof",
  proof_generated: "ZK proof",
  submitted: "Submitted on-chain",
  confirmed: "Confirmed on-chain",
};

function eventToStep(event: BackendIntentEvent): ExecutionStep {
  switch (event.kind) {
    case "parsed":
      return {
        status: "monitoring",
        label: EVENT_LABEL.parsed,
        detail: "Otter parsed your words and is watching the condition.",
      };
    case "condition_met":
      return {
        status: "condition_met",
        label: EVENT_LABEL.condition_met,
        detail: "The on-chain condition fired — execution starts.",
      };
    case "proof_started":
      return {
        status: "proving",
        label: EVENT_LABEL.proof_started,
        detail: "Generating the delegation proof with Noir/Barretenberg…",
      };
    case "proof_generated":
      return {
        status: "proving",
        label: EVENT_LABEL.proof_generated,
        detail: `Proof generated (${event.detail ?? "hash unavailable"})`,
      };
    case "submitted":
      return {
        status: "submitted",
        label: EVENT_LABEL.submitted,
        detail: event.detail ?? "Transaction submitted",
      };
    case "confirmed":
      return {
        status: "confirmed",
        label: EVENT_LABEL.confirmed,
        detail: event.detail ?? "Transaction confirmed",
      };
  }
}

/** Events → timeline steps. proof_started/proof_generated merge into one
 * "proving" step (generated updates it in place) so the timeline stays
 * linear and keys unique. */
function eventsToSteps(events: BackendIntentEvent[]): ExecutionStep[] {
  const steps: ExecutionStep[] = [];
  for (const event of events) {
    const step = eventToStep(event);
    step.timestamp = new Date(event.at * 1000).toISOString();
    if (event.kind === "proof_started" || event.kind === "proof_generated") {
      const existing = steps.findIndex((s) => s.status === "proving");
      if (existing >= 0) {
        steps[existing] = step;
        continue;
      }
    }
    steps.push(step);
  }
  return steps;
}

/**
 * Live execution timeline for an intent: the backend publishes lifecycle
 * events (parsed → condition met → proof → submitted → confirmed) on the
 * event bus and serves them from GET /api/v1/intents/:id/events. Older
 * intents (created before lifecycle events existed) fall back to the
 * executions table, which only records confirmed transactions.
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
    const fetchStatus = async () => {
      try {
        const eventsRes = await api.intents.events(intentId);
        if (!mounted) return;
        if (eventsRes.events.length > 0) {
          const steps = eventsToSteps(eventsRes.events);
          setData({
            intentId,
            currentStep: steps[steps.length - 1].status,
            steps,
            startedAt: steps[0].timestamp ?? "",
            updatedAt: steps[steps.length - 1].timestamp ?? "",
          });
          setError(null);
          return;
        }

        // Fallback for intents older than the lifecycle log: the executions
        // table only knows confirmed transactions.
        const res = await api.executions.list();
        if (!mounted) return;
        const records = res.executions
          .filter((record) => record.intent_id === intentId)
          .sort((a, b) => a.created_at - b.created_at);
        if (records.length === 0) {
          setData(null);
        } else {
          const steps = records.map(toExecutionStep);
          setData({
            intentId,
            currentStep: "confirmed",
            steps,
            startedAt: steps[0].timestamp ?? "",
            updatedAt: steps[steps.length - 1].timestamp ?? "",
          });
        }
        setError(null);
      } catch (err) {
        if (mounted) setError(err instanceof Error ? err : new Error(String(err)));
      } finally {
        if (mounted) setIsLoading(false);
      }
    };
    setIsLoading(true);
    fetchStatus();
    if (!poll)
      return () => {
        mounted = false;
      };
    const interval = setInterval(fetchStatus, 3000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [intentId, poll, isAuthenticated]);

  return { data, isLoading, error };
}
