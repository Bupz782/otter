import { useCallback, useEffect, useState } from "react";
import { Zap, Package, Percent } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { useAuthToken } from "@/hooks/useAuthToken";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { api, type BackendMevBundle } from "@/lib/api";
import { truncateHash } from "@/lib/utils";

function bundleStatusVariant(status: string): "default" | "secondary" | "outline" {
  if (status === "submitted") return "default";
  if (status === "failed") return "outline";
  return "secondary";
}

export function MevPage() {
  useDocumentTitle("MEV");
  const { isAuthenticated } = useAuthToken();

  const [bundles, setBundles] = useState<BackendMevBundle[] | null>(null);
  const [bundlesLoading, setBundlesLoading] = useState(true);
  const [bundlesError, setBundlesError] = useState<Error | null>(null);

  const [rebateBps, setRebateBps] = useState<number | null>(null);

  const fetchBundles = useCallback(async () => {
    setBundlesLoading(true);
    setBundlesError(null);
    try {
      setBundles(await api.mev.bundles());
    } catch (err) {
      setBundlesError(err instanceof Error ? err : new Error("Unknown error"));
    } finally {
      setBundlesLoading(false);
    }
  }, []);

  const fetchConfig = useCallback(async () => {
    try {
      const config = await api.mev.getConfig();
      setRebateBps(config.rebate_bps);
    } catch {
      setRebateBps(null);
    }
  }, []);

  useEffect(() => {
    if (isAuthenticated) {
      fetchBundles();
      fetchConfig();
    } else {
      setBundles(null);
      setBundlesLoading(false);
    }
  }, [isAuthenticated, fetchBundles, fetchConfig]);

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-6xl space-y-6">
        <PageHeader title="MEV" subtitle="Bundle submissions and rebate share." />
        <SectionCard>
          <EmptyState
            title="Sign in required"
            description="Connect your wallet to view bundle submissions and the rebate share."
          />
        </SectionCard>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <PageHeader title="MEV" subtitle="Bundle-based searcher: submissions and rebate share." />

      <SectionCard
        title="Rebate share"
        subtitle="Share of the MEV captured on your executions that the protocol pays back to you. Set by the platform operator — OTTER_MEV_REBATE_BPS at boot, runtime API override."
      >
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg border border-border bg-secondary">
            <Percent className="h-4 w-4 text-accent" />
          </div>
          {rebateBps === null ? (
            <Skeleton className="h-8 w-24" />
          ) : (
            <p className="font-heading text-2xl font-bold tabular-nums">
              {(rebateBps / 100).toLocaleString()}%
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                of captured MEV is rebated to you
              </span>
            </p>
          )}
        </div>
      </SectionCard>

      <SectionCard title="Submitted bundles" subtitle="Bundles sent to the private relay, most recent first.">
        {bundlesLoading ? (
          <Skeleton className="h-32 w-full" />
        ) : bundlesError ? (
          <ErrorState subject="bundle history" onRetry={fetchBundles} />
        ) : !bundles || bundles.length === 0 ? (
          <EmptyState
            title="No bundles yet"
            description="Bundles appear here once the backrun monitor detects a target transaction or a bundle is submitted manually."
          />
        ) : (
          <ul className="space-y-3">
            {bundles.map((b) => (
              <li
                key={b.bundle_hash}
                className="flex flex-wrap items-center gap-3 rounded-lg border border-border/60 bg-secondary p-4"
              >
                <div className="min-w-0 flex-1">
                  <p className="font-mono text-sm">{truncateHash(b.bundle_hash)}</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {b.target_tx_hash ? (
                      <>target <span className="font-mono">{truncateHash(b.target_tx_hash)}</span> · </>
                    ) : (
                      <>manual · </>
                    )}
                    {new Date(b.created_at * 1000).toLocaleString()}
                  </p>
                </div>
                <Badge variant={bundleStatusVariant(b.status)}>{b.status}</Badge>
              </li>
            ))}
          </ul>
        )}
        {!bundlesLoading && !bundlesError && bundles && bundles.length > 0 && (
          <div className="mt-4 flex justify-end">
            <Button onClick={fetchBundles} variant="outline">
              Refresh
            </Button>
          </div>
        )}
      </SectionCard>
    </div>
  );
}
