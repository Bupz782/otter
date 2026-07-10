import { useState } from "react";
import { api } from "@/lib/api";
import type { CreateStrategyPayload, Strategy } from "@/types/app";

export function useCreateStrategy() {
  const [data, setData] = useState<Strategy | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = async (payload: CreateStrategyPayload) => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await api.strategies.create(payload);
      const strategy: Strategy = {
        id: result.id,
        agentId: payload.agentId,
        agentName: payload.agentId,
        title: payload.title,
        description: payload.description,
        rawText: payload.rawText,
        riskProfile: payload.riskProfile,
        copies: 0,
        totalVolume: 0,
        apy: 0,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };
      setData(strategy);
      return strategy;
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to create strategy");
      setError(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  return { data, isLoading, error, mutate };
}
