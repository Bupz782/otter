import { useState } from "react";
import { api, mapBackendConditionalIntent } from "@/lib/api";
import type { ParsedIntent } from "@/types/app";

export function useParseIntent() {
  const [data, setData] = useState<ParsedIntent | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const parse = async (text: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await api.intents.parse(text);
      const mapped = mapBackendConditionalIntent(result.intent);
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
