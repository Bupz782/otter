import { useEffect, useState } from "react";
import { api, mapBackendProof } from "@/lib/api";
import type { Proof } from "@/types/app";

export function useProofs() {
  const [data, setData] = useState<Proof[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    setIsLoading(true);
    api.proofs
      .list()
      .then((res) => setData(res.proofs.map(mapBackendProof)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  }, []);

  return { data, isLoading, error };
}
