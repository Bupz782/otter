import { useEffect, useState } from "react";
import { Wallet, Percent } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { api, ApiClientError } from "@/lib/api";

interface RebateSummary {
  total_rebated_wei: string;
  rebate_bps: number;
}

export function RebatesPage() {
  useDocumentTitle("Rebates");
  const [summary, setSummary] = useState<RebateSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchSummary = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await api.rebates.list();
      setSummary(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSummary();
  }, []);

  const isUnavailable = error && error instanceof ApiClientError && error.status === 503;

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="MEV Rebates" subtitle="Profit shared back from captured MEV." />

      {loading ? (
        <Skeleton className="h-48 w-full" />
      ) : isUnavailable ? (
        <SectionCard>
          <EmptyState
            icon={<Wallet className="h-6 w-6" />}
            title="Rebates unavailable"
            description="MEV capture is not enabled on this backend."
          />
        </SectionCard>
      ) : error ? (
        <ErrorState subject="rebate summary" onRetry={fetchSummary} />
      ) : summary ? (
        <SectionCard title="Rebate summary" subtitle="Current period">
          <dl className="grid gap-4 sm:grid-cols-2">
            <div className="rounded-lg border border-border/60 bg-secondary p-4">
              <dt className="text-xs text-muted-foreground">Total rebated (wei)</dt>
              <dd className="mt-1 break-all font-mono text-lg">
                {summary.total_rebated_wei}
              </dd>
            </div>
            <div className="rounded-lg border border-border/60 bg-secondary p-4">
              <dt className="text-xs text-muted-foreground">Rebate share</dt>
              <dd className="mt-1 flex items-center gap-2 text-lg font-medium">
                <Percent className="h-4 w-4 text-muted-foreground" />
                {summary.rebate_bps / 100}%
              </dd>
            </div>
          </dl>
        </SectionCard>
      ) : null}

      {!loading && !isUnavailable && (
        <div className="flex justify-end">
          <Button onClick={fetchSummary} variant="outline">
            Refresh
          </Button>
        </div>
      )}
    </div>
  );
}
