import { useState, type ReactNode } from "react";
import { Link } from "react-router-dom";
import { ShieldCheck, CheckCircle2, XCircle, ChevronDown, ChevronUp } from "lucide-react";
import { useDocumentTitle } from "@/hooks/useDocumentTitle";
import { motion, AnimatePresence } from "framer-motion";
import { Skeleton } from "@/components/ui/skeleton";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/app/PageHeader";
import { SectionCard } from "@/components/app/SectionCard";
import { DataRow } from "@/components/app/DataRow";
import { EmptyState } from "@/components/app/EmptyState";
import { ErrorState } from "@/components/app/ErrorState";
import { DemoDataNotice } from "@/components/app/DemoDataNotice";
import { useProofs } from "@/hooks/useProofs";
import { cn } from "@/lib/utils";
import type { Proof } from "@/types/app";

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

const VERIFIED_CLASSES = "border-emerald-400/30 bg-emerald-400/10 text-emerald-400";
const INVALID_CLASSES = "border-rose-400/30 bg-rose-400/10 text-rose-400";

function VerifiedBadge({ verified }: { verified: boolean }) {
  return (
    <span
      className={cn(
        "inline-flex shrink-0 items-center rounded-full border px-2.5 py-0.5 text-xs",
        verified ? VERIFIED_CLASSES : INVALID_CLASSES
      )}
    >
      {verified ? "Verified" : "Invalid"}
    </span>
  );
}

function ProofRow({
  proof,
  isExpanded,
  onToggle,
}: {
  proof: Proof;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  return (
    <DataRow className="flex-wrap">
      <div
        className={cn(
          "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border",
          proof.verified ? VERIFIED_CLASSES : INVALID_CLASSES
        )}
      >
        {proof.verified ? <CheckCircle2 className="h-4 w-4" /> : <XCircle className="h-4 w-4" />}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium capitalize">{proof.type} proof</p>
        <p className="truncate text-xs text-muted-foreground">
          {proof.verifier} · {proof.constraints.toLocaleString()} constraints · {proof.proofTime}s
        </p>
      </div>
      <VerifiedBadge verified={proof.verified} />
      <Button
        variant="ghost"
        size="sm"
        onClick={onToggle}
        aria-expanded={isExpanded}
        aria-label={isExpanded ? "Hide proof details" : "Show proof details"}
      >
        {isExpanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
      </Button>
      <AnimatePresence initial={false}>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="w-full overflow-hidden"
          >
            <dl className="mt-1 grid gap-3 border-t border-border/40 pt-3 sm:grid-cols-2">
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Type</dt>
                <dd className="mt-1 font-mono text-sm capitalize">{proof.type}</dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Circuit</dt>
                <dd className="mt-1 font-mono text-sm">{proof.verifier}</dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Constraints</dt>
                <dd className="mt-1 font-mono text-sm tabular-nums">
                  {proof.constraints.toLocaleString()}
                </dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3">
                <dt className="text-xs text-muted-foreground">Generation time</dt>
                <dd className="mt-1 font-mono text-sm tabular-nums">{proof.proofTime}s</dd>
              </div>
              <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
                <dt className="text-xs text-muted-foreground">Timestamp</dt>
                <dd className="mt-1 font-mono text-sm">
                  {new Date(proof.timestamp).toLocaleString()}
                </dd>
              </div>
              {proof.intentId && (
                <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
                  <dt className="text-xs text-muted-foreground">Intent</dt>
                  <dd className="mt-1 text-sm">
                    <Link
                      to={`/app/intents/${proof.intentId}`}
                      className="text-accent hover:underline"
                    >
                      View intent
                    </Link>
                  </dd>
                </div>
              )}
              {proof.txHash && (
                <div className="rounded-lg border border-border/60 bg-secondary p-3 sm:col-span-2">
                  <dt className="text-xs text-muted-foreground">Tx hash</dt>
                  <dd className="mt-1 break-all font-mono text-xs">{proof.txHash}</dd>
                </div>
              )}
            </dl>
          </motion.div>
        )}
      </AnimatePresence>
    </DataRow>
  );
}

export function ProofsPage() {
  useDocumentTitle("Proofs");
  const { data: proofs, isLoading, error, refetch, isBackendDemo } = useProofs();
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const latestSolvency = proofs
    .filter((proof) => proof.type === "solvency")
    .sort((a, b) => Date.parse(b.timestamp) - Date.parse(a.timestamp))[0];

  const toggle = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  return (
    <div className="mx-auto max-w-6xl space-y-6">
      <FadeIn>
        <PageHeader title="Proofs" subtitle="Every execution, proven." />
      </FadeIn>

      {isBackendDemo && (
        <FadeIn delay={0.05}>
          <DemoDataNotice />
        </FadeIn>
      )}

      <FadeIn delay={0.05}>
        {isLoading ? (
          <Skeleton className="h-24 w-full" />
        ) : (
          latestSolvency && (
            <SectionCard className="py-4">
              <div className="flex items-center gap-4">
                <div
                  className={cn(
                    "flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border",
                    latestSolvency.verified ? VERIFIED_CLASSES : INVALID_CLASSES
                  )}
                >
                  {latestSolvency.verified ? (
                    <CheckCircle2 className="h-4 w-4" />
                  ) : (
                    <XCircle className="h-4 w-4" />
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="font-heading text-base font-bold">Vault solvency</p>
                  <p className="text-sm text-muted-foreground">
                    Latest proof {latestSolvency.verified ? "passed" : "failed"}{" "}
                    {new Date(latestSolvency.timestamp).toLocaleString()}.
                  </p>
                </div>
                <VerifiedBadge verified={latestSolvency.verified} />
              </div>
            </SectionCard>
          )
        )}
      </FadeIn>

      <FadeIn delay={0.1}>
        <SectionCard title="Recent proofs" subtitle="Delegation, execution, and solvency proofs.">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
              <Skeleton className="h-16 w-full" />
            </div>
          ) : error ? (
            <ErrorState subject="proofs" onRetry={refetch} />
          ) : proofs.length === 0 ? (
            <EmptyState
              icon={<ShieldCheck className="h-6 w-6" />}
              title="No proofs yet"
              description="When an intent executes, its proof lands here."
            />
          ) : (
            <div className="space-y-3">
              {proofs.map((proof) => (
                <ProofRow
                  key={proof.id}
                  proof={proof}
                  isExpanded={expanded.has(proof.id)}
                  onToggle={() => toggle(proof.id)}
                />
              ))}
            </div>
          )}
        </SectionCard>
      </FadeIn>
    </div>
  );
}
