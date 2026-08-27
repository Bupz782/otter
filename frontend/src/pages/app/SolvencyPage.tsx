import { useEffect, useState } from "react";
import { ShieldCheck, AlertCircle } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { api, ApiClientError } from "@/lib/api";

interface SolvencyStatus {
  registry?: string;
  merkle_root?: string;
  total_deposits_wei?: string;
  last_proven_at?: number;
}

export function SolvencyPage() {
  useDocumentTitle("Solvency");
  const [status, setStatus] = useState<SolvencyStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.solvency.status();
      setStatus(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const isUnavailable = error && error instanceof ApiClientError && error.status === 503;

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="Solvency" subtitle="On-chain proof-of-solvency status." />

      {loading ? (
        <Skeleton className="h-48 w-full" />
      ) : isUnavailable ? (
        <SectionCard>
          <EmptyState
            icon={<AlertCircle className="h-6 w-6" />}
            title="Solvency registry not configured"
            description="Set OTTER_SOLVENCY_REGISTRY on the backend to enable live solvency reads."
          />
        </SectionCard>
      ) : error ? (
        <ErrorState subject="solvency status" onRetry={fetchStatus} />
      ) : status && status.registry ? (
        <SectionCard title="Registry state" subtitle={`Registry: ${status.registry}`}>
          <dl className="grid gap-4 sm:grid-cols-2">
            <div className="rounded-lg border border-border/60 bg-secondary p-4">
              <dt className="text-xs text-muted-foreground">Merkle root</dt>
              <dd className="mt-1 break-all font-mono text-sm">
                {status.merkle_root ?? "—"}
              </dd>
            </div>
            <div className="rounded-lg border border-border/60 bg-secondary p-4">
              <dt className="text-xs text-muted-foreground">Total deposits (wei)</dt>
              <dd className="mt-1 break-all font-mono text-sm">
                {status.total_deposits_wei ?? "—"}
              </dd>
            </div>
            <div className="rounded-lg border border-border/60 bg-secondary p-4 sm:col-span-2">
              <dt className="text-xs text-muted-foreground">Last proven at</dt>
              <dd className="mt-1 text-sm">
                {status.last_proven_at
                  ? new Date(status.last_proven_at * 1000).toLocaleString()
                  : "—"}
              </dd>
            </div>
          </dl>
        </SectionCard>
      ) : (
        <SectionCard>
          <EmptyState
            icon={<ShieldCheck className="h-6 w-6" />}
            title="No solvency data"
            description="The registry is configured but has not been proven yet."
          />
        </SectionCard>
      )}

      {!loading && !isUnavailable && (
        <div className="flex justify-end">
          <Button onClick={fetchStatus} variant="outline">
            Refresh
          </Button>
        </div>
      )}
    </div>
  );
}
