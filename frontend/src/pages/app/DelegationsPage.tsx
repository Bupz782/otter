import type { ReactNode } from "react";
import { Link } from "react-router-dom";
import { Plus, FileSignature } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { DataRow } from "@/components/app/DataRow";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { useDelegations } from "@/hooks/useDelegations";
import { useIntents } from "@/hooks/useIntents";
import { getStatusPresentation, type StatusPresentation } from "@/lib/status";
import { truncateHash, cn } from "@/lib/utils";
import type { Delegation } from "@/types/app";

const EASE: [number, number, number, number] = [0.22, 1, 0.36, 1];

/** Mount-only fade/slide used to stagger the page blocks. */
function FadeIn({
  children,
  delay = 0,
  className,
}: {
  children: ReactNode;
  delay?: number;
  className?: string;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, ease: EASE, delay }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

/**
 * Delegation records only carry a status on demo fixtures; real backend
 * records are just hash + createdAt, and a returned delegation is active by
 * definition (no revoke endpoint exists), so it shows a muted "Active".
 */
function delegationStatus(delegation: Delegation): StatusPresentation {
  switch (delegation.status) {
    case "revoked":
      return getStatusPresentation("revoked");
    case "expired":
      return { ...getStatusPresentation("revoked"), label: "Expired" };
    case "active":
      return { ...getStatusPresentation("confirmed"), label: "Active" };
    default:
      return { ...getStatusPresentation("revoked"), label: "Active" };
  }
}

export function DelegationsPage() {
  useDocumentTitle("Delegations");
  const { data: delegations, isLoading, error, refetch, isDemo } = useDelegations();
  // Intents carry the delegation hash they run under — linked both ways.
  const { data: intents } = useIntents();

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Delegations"
          subtitle="Your agent, on a leash."
          action={
            <Button asChild className="rounded-full">
              <Link to="/app/delegations/new">
                <Plus className="mr-2 h-4 w-4" />
                New delegation
              </Link>
            </Button>
          }
        />
      </FadeIn>

      {isDemo && (
        <FadeIn delay={0.03}>
          <SectionCard className="py-4">
            <p className="text-sm text-muted-foreground">
              <span className="font-medium text-foreground">Demo data.</span> These delegations
              are pre-made examples so you can explore. Nothing here is yours yet — your first
              real step is{" "}
              <Link
                to="/app/delegations/new"
                className="font-medium text-accent underline underline-offset-2"
              >
                creating your own delegation
              </Link>
              : you sign the limits (amounts, protocols, expiry), and the agent can only act
              inside them.
            </p>
          </SectionCard>
        </FadeIn>
      )}

      <FadeIn delay={0.05}>
        <SectionCard
          title="Signed delegations"
          subtitle="An agent executes only inside the limits you sign."
        >
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
            </div>
          ) : error ? (
            <ErrorState subject="delegations" onRetry={refetch} />
          ) : delegations.length === 0 ? (
            <EmptyState
              icon={<FileSignature className="h-6 w-6" />}
              title="No delegations yet"
              description="Sign a delegation and an Otter agent starts working inside your limits."
              action={
                <Button asChild className="rounded-full">
                  <Link to="/app/delegations/new">Create your first delegation</Link>
                </Button>
              }
            />
          ) : (
            <div className="space-y-3">
              {delegations.map((delegation) => {
                const status = delegationStatus(delegation);
                return (
                  <DataRow key={delegation.id}>
                    <div className="w-32 shrink-0">
                      <p className="truncate font-mono text-sm">{truncateHash(delegation.id)}</p>
                      <p className="text-xs text-muted-foreground">
                        Signed {new Date(delegation.createdAt).toLocaleDateString()}
                      </p>
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium">
                        {delegation.agentName ?? "Otter Agent"}
                      </p>
                      {(delegation.allowedProtocols || delegation.allowedChains) && (
                        <div className="mt-1.5 flex flex-wrap gap-1.5">
                          {delegation.allowedProtocols?.map((protocol) => (
                            <Badge key={protocol} variant="outline">
                              {protocol}
                            </Badge>
                          ))}
                          {delegation.allowedChains?.map((chain) => (
                            <Badge key={chain} variant="outline">
                              {chain}
                            </Badge>
                          ))}
                        </div>
                      )}
                      {(() => {
                        const underThis = (intents ?? []).filter(
                          (intent) => intent.delegationId === delegation.id
                        );
                        if (underThis.length === 0) return null;
                        return (
                          <p className="mt-1.5 text-xs text-muted-foreground">
                            {underThis.length} intent{underThis.length > 1 ? "s" : ""} running
                            under this delegation:{" "}
                            {underThis.slice(0, 3).map((intent, index) => (
                              <span key={intent.id}>
                                {index > 0 && " · "}
                                <Link
                                  to={`/app/intents/${intent.id}`}
                                  className="text-accent underline underline-offset-2"
                                >
                                  {intent.rawText.length > 40
                                    ? `${intent.rawText.slice(0, 40)}…`
                                    : intent.rawText}
                                </Link>
                              </span>
                            ))}
                          </p>
                        );
                      })()}
                    </div>
                    <span
                      className={cn(
                        "inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs",
                        status.badgeClass
                      )}
                    >
                      <span className={cn("h-1.5 w-1.5 rounded-full", status.dotClass)} />
                      {status.label}
                    </span>
                    {/* No Revoke button: the backend has no revoke/delete
                        delegation endpoint (only GET/POST /api/v1/delegation in
                        crates/interfaces/src/bin/otter_api.rs). Backend follow-up. */}
                  </DataRow>
                );
              })}
            </div>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
