import { useCallback, useEffect, useState } from "react";
import { Zap, Package } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { useAuthToken } from "@/hooks/useAuthToken";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
  const [rebateDraft, setRebateDraft] = useState("");
  const [savingConfig, setSavingConfig] = useState(false);
  const [configError, setConfigError] = useState<string | null>(null);

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
      setRebateDraft(String(config.rebate_bps));
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

  const handleSaveConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    const value = Number(rebateDraft);
    if (!Number.isInteger(value) || value < 0 || value > 10_000) {
      setConfigError("Rebate must be an integer between 0 and 10000 bps.");
      return;
    }
    setSavingConfig(true);
    setConfigError(null);
    try {
      const updated = await api.mev.setConfig(value);
      setRebateBps(updated.rebate_bps);
    } catch (err) {
      setConfigError(err instanceof Error ? err.message : "Could not save the rebate share.");
    } finally {
      setSavingConfig(false);
    }
  };

  if (!isAuthenticated) {
    return (
      <div className="mx-auto max-w-6xl space-y-6">
        <PageHeader title="MEV" subtitle="Bundle submissions and rebate configuration." />
        <SectionCard>
          <EmptyState
            icon={<Zap className="h-6 w-6" />}
            title="Sign in required"
            description="Connect your wallet and sign in to view bundle submissions and tune the rebate share."
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
        subtitle="Share of captured profit rebated to the vault owner, in basis points. Runtime override; the boot value comes from OTTER_MEV_REBATE_BPS."
      >
        <form onSubmit={handleSaveConfig} className="flex flex-wrap items-end gap-3">
          <div className="space-y-2">
            <Label htmlFor="rebate-bps">Rebate (bps)</Label>
            <Input
              id="rebate-bps"
              type="number"
              min={0}
              max={10000}
              value={rebateDraft}
              onChange={(e) => setRebateDraft(e.target.value)}
              placeholder={rebateBps !== null ? String(rebateBps) : "5000"}
              className="w-40"
              required
            />
          </div>
          <Button type="submit" disabled={savingConfig}>
            {savingConfig ? "Saving…" : "Save"}
          </Button>
          {rebateBps !== null && (
            <p className="text-sm text-muted-foreground">Current: {rebateBps / 100}%</p>
          )}
        </form>
        {configError && <p className="mt-3 text-sm text-destructive">{configError}</p>}
      </SectionCard>

      <SectionCard title="Submitted bundles" subtitle="Bundles sent to the private relay, most recent first.">
        {bundlesLoading ? (
          <Skeleton className="h-32 w-full" />
        ) : bundlesError ? (
          <ErrorState subject="bundle history" onRetry={fetchBundles} />
        ) : !bundles || bundles.length === 0 ? (
          <EmptyState
            icon={<Package className="h-6 w-6" />}
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
