import { useState } from "react";
import { api } from "@/lib/api";
import type { IntentType } from "@/types/app";

export function useParseIntent() {
  const [data, setData] = useState<{
    type: IntentType;
    amount: number;
    asset: string;
    protocol: string;
    condition?: string;
    chain: string;
  } | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const parse = async (text: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await api.intents.parse(text);
      const parsed = result.intent as {
        type?: string;
        amount?: number;
        asset?: string;
        protocol?: string;
        condition?: string;
        chain?: string;
        action?: string;
      };
      const mapped = {
        type:
          (parsed.type?.toLowerCase() as IntentType) ||
          (parsed.action?.toLowerCase() as IntentType) ||
          "lend",
        amount: parsed.amount || 0,
        asset: parsed.asset || "USDC",
        protocol: parsed.protocol || "Aave",
        condition: parsed.condition,
        chain: parsed.chain || "Ethereum",
      };
      setData(mapped);
      return mapped;
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Parse failed"));
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  return { data, isLoading, error, parse, reset: () => setData(null) };
}
