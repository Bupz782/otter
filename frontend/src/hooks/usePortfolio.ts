import { useEffect, useState } from "react";
import { useAccount } from "wagmi";
import { api, mapBackendPortfolio } from "@/lib/api";
import type { Portfolio } from "@/types/app";

export function usePortfolio() {
  const { address } = useAccount();
  const [data, setData] = useState<Portfolio | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchPortfolio = () => {
    setIsLoading(true);
    api.portfolio
      .get()
      .then((res) => setData(mapBackendPortfolio(res)))
      .catch(setError)
      .finally(() => setIsLoading(false));
  };

  useEffect(() => {
    fetchPortfolio();
  }, [address]);

  return { data, isLoading, error, refetch: fetchPortfolio };
}
