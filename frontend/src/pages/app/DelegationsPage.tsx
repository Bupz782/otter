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
  const { data: delegations, isLoading, error, refetch } = useDelegations();

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader
          title="Delegations"
          subtitle="Your agents, on a leash."
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
                        {delegation.agentName ?? "Agent not on record"}
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
